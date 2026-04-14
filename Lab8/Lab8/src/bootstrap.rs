use std::net::UdpSocket;
use std::io;
use std::time::Duration;

const TIMEOUT_SECS: u64 = 5;
const MAX_RETRIES: u32 = 3;

// Registers this peer with the bootstrap server.
// Returns Ok(Some((ip, port))) if an existing peer was returned,
// Ok(None) if this is the first peer, or an error.
pub fn register(
    own_addr: &str,
    own_port: u16,
    bootstrap_ip: &str,
    bootstrap_port: u16,
) -> io::Result<Option<(String, u16)>> {
    let socket = UdpSocket::bind("[::]:0")?;
    socket.set_read_timeout(Some(Duration::from_secs(TIMEOUT_SECS)))?;

    let bootstrap_addr = format!("[{}]:{}", bootstrap_ip, bootstrap_port);
    let message = format!("REG {} {}", own_addr, own_port);

    println!("Sending to bootstrap: {}", message);

    // Retry loop in case of packet loss
    for attempt in 1..=MAX_RETRIES {
        socket.send_to(message.as_bytes(), &bootstrap_addr)?;

        let mut buf = [0u8; 1024];
        match socket.recv_from(&mut buf) {
            Ok((n, _)) => {
                let response = String::from_utf8_lossy(&buf[..n]).to_string();
                println!("Bootstrap response: {}", response);
                match parse_regok(&response) {
                    Ok(result) => return Ok(result),
                    Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {
                        return Ok(None);
                    }
                    Err(e) => return Err(e),
                }
            }
            Err(e) if e.kind() == io::ErrorKind::WouldBlock
                   || e.kind() == io::ErrorKind::TimedOut => {
                eprintln!("Attempt {}/{} timed out, retrying...", attempt, MAX_RETRIES);
            }
            Err(e) => return Err(e),
        }
    }

    Err(io::Error::new(
        io::ErrorKind::TimedOut,
        "Bootstrap did not respond after max retries",
    ))
}

// Unregisters this peer from the bootstrap server on leave.
pub fn unregister(
    own_addr: &str,
    own_port: u16,
    bootstrap_ip: &str,
    bootstrap_port: u16,
) -> io::Result<()> {
    let socket = UdpSocket::bind("[::]:0")?;
    socket.set_read_timeout(Some(Duration::from_secs(TIMEOUT_SECS)))?;

    let bootstrap_addr = format!("[{}]:{}", bootstrap_ip, bootstrap_port);
    let message = format!("UNREG {} {}", own_addr, own_port);

    println!("Sending to bootstrap: {}", message);

    for attempt in 1..=MAX_RETRIES {
        socket.send_to(message.as_bytes(), &bootstrap_addr)?;

        let mut buf = [0u8; 1024];
        match socket.recv_from(&mut buf) {
            Ok((n, _)) => {
                let response = String::from_utf8_lossy(&buf[..n]).to_string();
                println!("Bootstrap response: {}", response);
                if response.starts_with("UNREGOK") {
                    return Ok(());
                } else {
                    return Err(io::Error::new(
                        io::ErrorKind::Other,
                        format!("Unexpected bootstrap response: {}", response),
                    ));
                }
            }
            Err(e) if e.kind() == io::ErrorKind::WouldBlock
                   || e.kind() == io::ErrorKind::TimedOut => {
                eprintln!("Attempt {}/{} timed out, retrying...", attempt, MAX_RETRIES);
            }
            Err(e) => return Err(e),
        }
    }

    Err(io::Error::new(
        io::ErrorKind::TimedOut,
        "Bootstrap did not respond to UNREG after max retries",
    ))
}

// Parses REGOK response from bootstrap.
// "REGOK 0" -> Ok(None) first peer
// "REGOK 1 ip port" -> Ok(Some((ip, port))) existing peer
fn parse_regok(response: &str) -> io::Result<Option<(String, u16)>> {
    let parts: Vec<&str> = response.split_whitespace().collect();
    if response.contains("ERROR 2") {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "Already registered",
        ));
    }

    if parts.is_empty() || parts[0] != "REGOK" {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Expected REGOK, got: {}", response),
        ));
    }

    match parts.get(1).map(|s| *s) {
        Some("0") => Ok(None),
        Some("1") => {
            let ip = parts.get(2).ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidData, "Missing IP in REGOK 1")
            })?;
            let port = parts.get(3).ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidData, "Missing port in REGOK 1")
            })?.parse::<u16>().map_err(|_| {
                io::Error::new(io::ErrorKind::InvalidData, "Invalid port in REGOK 1")
            })?;
            Ok(Some((ip.to_string(), port)))
        }
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Unknown REGOK format: {}", response),
        )),
    }
}