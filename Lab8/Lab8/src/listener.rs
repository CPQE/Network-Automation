use std::net::UdpSocket;
use std::sync::{Arc, Mutex};
use crate::peer::{PeerState, send_message, parse_addr_with_interface};
use crate::query::handle_file_query;
use crate::transfer::handle_file_found;

// Binds to own port and loops forever handling incoming UDP messages from peers
pub fn run_listener(
    state: Arc<Mutex<PeerState>>,
    own_ip: &str,
    interface: &str,
    port: u16,
) {
    let bind_addr = format!("[{}%{}]:{}", own_ip, interface, port);
    let socket = UdpSocket::bind(&bind_addr)
        .unwrap_or_else(|_| {
            // Fall back to binding without interface scope if above fails
            let fallback = format!("[{}]:{}", own_ip, port);
            UdpSocket::bind(&fallback).expect("Failed to bind listener socket")
        });

    println!("Listener bound to {}", bind_addr);

    let mut buf = [0u8; 4096];

    loop {
        match socket.recv_from(&mut buf) {
            Ok((n, src)) => {
                let msg = String::from_utf8_lossy(&buf[..n]).trim().to_string();
                println!("[Listener] From {}: {}", src, msg);
                handle_message(Arc::clone(&state), &msg, &socket, src);
            }
            Err(e) => {
                eprintln!("[Listener] recv error: {}", e);
            }
        }
    }
}

// Dispatches incoming messages to the appropriate handler
fn handle_message(
    state: Arc<Mutex<PeerState>>,
    msg: &str,
    socket: &UdpSocket,
    src: std::net::SocketAddr
) {
    let parts: Vec<&str> = msg.split_whitespace().collect();
    if parts.is_empty() {
        return;
    }
    match parts[0] {
        // Another peer is joining and wants our successor
        "JOIN" => handle_join(state, &parts, socket, src),
        // Update our successor to the given peer
        "SET_SUCCESSOR" => handle_set_successor(state, &parts),
        // Update our predecessor to the given peer
        "SET_PREDECESSOR" => handle_set_predecessor(state, &parts),
        // Incoming file query — check own files or forward to successor
        "FILE_QUERY" => handle_file_query(state, &parts),
        // A peer found the file we were looking for — initiate download
        "FILE_FOUND" => handle_file_found(state, &parts),
        _ => {
            eprintln!("[Listener] Unknown message: {}", msg);
        }
    }
}

// Handles JOIN from a new peer.
// Responds with JOINOK containing our current successor,
// our successor will be updated to the new peer by SET_SUCCESSOR.
fn handle_join(
    state: Arc<Mutex<PeerState>>,
    parts: &[&str],
    socket: &UdpSocket,
    src: std::net::SocketAddr
) {
    // Format: JOIN ip%interface port
    if parts.len() < 3 {
        eprintln!("[Listener] Malformed JOIN: {:?}", parts);
        return;
    }
    let (successor_addr, successor_port) = {
        let s: std::sync::MutexGuard<'_, PeerState> = state.lock().unwrap();
        (s.successor_addr(), s.successor_port)
    };

    let joiner_addr = parts[1];
    let joiner_port: u16 = match parts[2].parse() {
        Ok(p) => p,
        Err(_) => {
            eprintln!("[Listener] Invalid port in JOIN");
            return;
        }
    };

    // Respond with our current successor so the joiner can slot in between us
    let response = format!("JOINOK {} {}", successor_addr, successor_port);
    println!("[Listener] Sending JOINOK to {}: {}", src, response);

    // Send back to actual source address of the packet, not constructed address
    if let Err(e) = socket.send_to(response.as_bytes(), src) {
        eprintln!("[Listener] Failed to send JOINOK: {}", e);
    }
    // Parse clean IP for sending
    let joiner_ip_clean = joiner_addr.split('%').next().unwrap_or(joiner_addr);
    if let Err(e) = send_message(joiner_ip_clean, joiner_port, &response) {
        eprintln!("[Listener] Failed to send JOINOK: {}", e);
    }
}

// Handles SET_SUCCESSOR — updates our successor to the given peer
fn handle_set_successor(state: Arc<Mutex<PeerState>>, parts: &[&str]) {
    // Format: SET_SUCCESSOR ip%interface port
    if parts.len() < 3 {
        eprintln!("[Listener] Malformed SET_SUCCESSOR: {:?}", parts);
        return;
    }

    let addr = parts[1];
    let port: u16 = match parts[2].parse() {
        Ok(p) => p,
        Err(_) => {
            eprintln!("[Listener] Invalid port in SET_SUCCESSOR");
            return;
        }
    };

    let (ip, iface) = parse_addr_with_interface(addr);
    let mut s = state.lock().unwrap();
    s.successor_ip = ip;
    s.successor_interface = iface;
    s.successor_port = port;

    println!("[Listener] Updated successor to {}:{}", s.successor_addr(), s.successor_port);
    s.print_state();
}

// Handles SET_PREDECESSOR — updates our predecessor to the given peer
fn handle_set_predecessor(state: Arc<Mutex<PeerState>>, parts: &[&str]) {
    // Format: SET_PREDECESSOR ip%interface port
    if parts.len() < 3 {
        eprintln!("[Listener] Malformed SET_PREDECESSOR: {:?}", parts);
        return;
    }

    let addr = parts[1];
    let port: u16 = match parts[2].parse() {
        Ok(p) => p,
        Err(_) => {
            eprintln!("[Listener] Invalid port in SET_PREDECESSOR");
            return;
        }
    };

    let (ip, iface) = parse_addr_with_interface(addr);
    let mut s = state.lock().unwrap();
    s.predecessor_ip = ip;
    s.predecessor_interface = iface;
    s.predecessor_port = port;

    println!("[Listener] Updated predecessor to {}:{}", s.predecessor_addr(), s.predecessor_port);
    s.print_state();
}