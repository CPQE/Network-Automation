use std::net::UdpSocket;
use std::sync::{Arc, Mutex};
use std::io;
use std::time::Duration;

const TIMEOUT_SECS: u64 = 5;
const MAX_RETRIES: u32 = 3;

// Holds the full state of this peer node
pub struct PeerState {
    // Own identity
    pub own_ip: String,
    pub own_interface: String,
    pub own_port: u16,

    // Successor
    pub successor_ip: String,
    pub successor_interface: String,
    pub successor_port: u16,

    // Predecessor
    pub predecessor_ip: String,
    pub predecessor_interface: String,
    pub predecessor_port: u16,
}

impl PeerState {
    // Initializes peer with successor and predecessor pointing to self
    pub fn new(own_ip: String, own_interface: String, own_port: u16) -> Self {
        PeerState {
            successor_ip: own_ip.clone(),
            successor_interface: own_interface.clone(),
            successor_port: own_port,
            predecessor_ip: own_ip.clone(),
            predecessor_interface: own_interface.clone(),
            predecessor_port: own_port,
            own_ip,
            own_interface,
            own_port,
        }
    }

    // Formats own address as ip%interface for messages
    pub fn own_addr(&self) -> String {
        format!("{}%{}", self.own_ip, self.own_interface)
    }

    // Formats successor address as ip%interface
    pub fn successor_addr(&self) -> String {
        format!("{}%{}", self.successor_ip, self.successor_interface)
    }

    // Formats predecessor address as ip%interface
    pub fn predecessor_addr(&self) -> String {
        format!("{}%{}", self.predecessor_ip, self.predecessor_interface)
    }

    // Prints current routing state to stdout
    pub fn print_state(&self) {
        println!("=== Peer State ===");
        println!("  Own:         {}:{}", self.own_addr(), self.own_port);
        println!("  Successor:   {}:{}", self.successor_addr(), self.successor_port);
        println!("  Predecessor: {}:{}", self.predecessor_addr(), self.predecessor_port);
        println!("==================");
    }

    // Returns true if this peer is the only node in the ring
    pub fn is_alone(&self) -> bool {
        self.successor_ip == self.own_ip && self.successor_port == self.own_port
    }
}

// Sends a UDP message to a peer and waits for a response
fn send_and_receive(
    target_ip: &str,
    target_port: u16,
    message: &str,
) -> io::Result<String> {
    let socket = UdpSocket::bind("[::]:0")?;
    socket.set_read_timeout(Some(Duration::from_secs(TIMEOUT_SECS)))?;

    let target_addr = format!("[{}]:{}", target_ip, target_port);
    println!("Sending to {}: {}", target_addr, message);

    for attempt in 1..=MAX_RETRIES {
        socket.send_to(message.as_bytes(), &target_addr)?;
        let mut buf = [0u8; 1024];
        match socket.recv_from(&mut buf) {
            Ok((n, _)) => {
                let response = String::from_utf8_lossy(&buf[..n]).to_string();
                println!("Response from {}: {}", target_addr, response);
                return Ok(response);
            }
            Err(e) if e.kind() == io::ErrorKind::WouldBlock
                   || e.kind() == io::ErrorKind::TimedOut => {
                eprintln!("Attempt {}/{} timed out", attempt, MAX_RETRIES);
            }
            Err(e) => return Err(e),
        }
    }

    Err(io::Error::new(io::ErrorKind::TimedOut, "Peer did not respond"))
}

// Sends a fire-and-forget UDP message to a peer with no response expected
pub fn send_message(target_ip: &str, target_port: u16, message: &str) -> io::Result<()> {
    let socket = UdpSocket::bind("[::]:0")?;
    let target_addr = format!("[{}]:{}", target_ip, target_port);
    println!("Sending to {}: {}", target_addr, message);
    socket.send_to(message.as_bytes(), &target_addr)?;
    Ok(())
}

// Joins the ring by contacting an existing peer.
// Sends JOIN, receives JOINOK with existing peer's successor,
// then sends SET_SUCCESSOR and SET_PREDECESSOR to update neighbors.
pub fn join_ring(
    state: Arc<Mutex<PeerState>>,
    existing_ip: &str,
    existing_port: u16,
) -> io::Result<()> {
    let (own_addr, own_port) = {
        let s = state.lock().unwrap();
        (s.own_addr(), s.own_port)
    };

    // Strip %interface from existing_ip if present for socket connection
    let existing_ip_clean = existing_ip.split('%').next().unwrap_or(existing_ip);

    // Send JOIN to existing peer
    let join_msg = format!("JOIN {} {}", own_addr, own_port);
    let response = send_and_receive(existing_ip_clean, existing_port, &join_msg)?;

    // Parse JOINOK response — contains existing peer's current successor
    // Format: "JOINOK ip%interface port"
    let parts: Vec<&str> = response.split_whitespace().collect();
    if parts.is_empty() || parts[0] != "JOINOK" {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Expected JOINOK, got: {}", response),
        ));
    }

    let successor_addr = parts.get(1).ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidData, "Missing successor addr in JOINOK")
    })?;
    let successor_port = parts.get(2).ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidData, "Missing successor port in JOINOK")
    })?.parse::<u16>().map_err(|_| {
        io::Error::new(io::ErrorKind::InvalidData, "Invalid successor port in JOINOK")
    })?;

    // Parse ip%interface from successor_addr
    let (successor_ip, successor_iface) = parse_addr_with_interface(successor_addr);

    // Update own state
    {
        let mut s = state.lock().unwrap();
        s.successor_ip = successor_ip.clone();
        s.successor_interface = successor_iface.clone();
        s.successor_port = successor_port;
        s.predecessor_ip = existing_ip_clean.to_string();
        s.predecessor_interface = existing_ip.split('%').nth(1)
            .unwrap_or("enp7s0").to_string();
        s.predecessor_port = existing_port;
    }

    // Notify our new successor to update its predecessor to us
    let successor_ip_clean = successor_ip.split('%').next().unwrap_or(&successor_ip).to_string();
    let set_pred_msg = format!("SET_PREDECESSOR {} {}", own_addr, own_port);
    send_message(&successor_ip_clean, successor_port, &set_pred_msg)?;

    // Notify existing peer to update its successor to us
    let set_succ_msg = format!("SET_SUCCESSOR {} {}", own_addr, own_port);
    send_message(existing_ip_clean, existing_port, &set_succ_msg)?;

    println!("Successfully joined ring");
    {
        state.lock().unwrap().print_state();
    }

    Ok(())
}

// Handles a leaving peer — notifies successor and predecessor to relink,
// then unregisters from bootstrap
pub fn leave_ring(state: Arc<Mutex<PeerState>>) -> io::Result<()> {
    let s = state.lock().unwrap();

    if s.is_alone() {
        println!("Only peer in network, nothing to notify");
        return Ok(());
    }

    let successor_ip_clean = s.successor_ip.split('%').next()
        .unwrap_or(&s.successor_ip).to_string();
    let predecessor_ip_clean = s.predecessor_ip.split('%').next()
        .unwrap_or(&s.predecessor_ip).to_string();

    // Tell successor its new predecessor is our predecessor
    let msg = format!("SET_PREDECESSOR {} {}", s.predecessor_addr(), s.predecessor_port);
    send_message(&successor_ip_clean, s.successor_port, &msg)?;

    // Tell predecessor its new successor is our successor
    let msg = format!("SET_SUCCESSOR {} {}", s.successor_addr(), s.successor_port);
    send_message(&predecessor_ip_clean, s.predecessor_port, &msg)?;

    println!("Notified successor and predecessor of departure");
    Ok(())
}

// Parses "ip%interface" into ("ip", "interface")
// If no % present, returns ("ip", "enp7s0") as default
pub fn parse_addr_with_interface(addr: &str) -> (String, String) {
    if let Some(idx) = addr.find('%') {
        (addr[..idx].to_string(), addr[idx+1..].to_string())
    } else {
        (addr.to_string(), "enp7s0".to_string())
    }
}