use std::sync::{Arc, Mutex};
use crate::peer::{PeerState, send_message};
use crate::files::{file_exists, list_files};

pub fn handle_file_query(state: Arc<Mutex<PeerState>>, parts: &[&str]) {
    if parts.len() < 5 {
        eprintln!("[Query] Malformed FILE_QUERY: {:?}", parts);
        return;
    }

    let hop_count: u32 = match parts[1].parse() {
        Ok(h) => h,
        Err(_) => {
            eprintln!("[Query] Invalid hop count");
            return;
        }
    };

    let filename     = parts[2];
    let source_addr  = parts[3];
    let source_port: u16 = match parts[4].parse() {
        Ok(p) => p,
        Err(_) => {
            eprintln!("[Query] Invalid source port");
            return;
        }
    };

    println!("[Query] FILE_QUERY for '{}' hop_count={} from {}:{}",
             filename, hop_count, source_addr, source_port);

    if file_exists(filename) {
        println!("[Query] File '{}' found locally, notifying requester", filename);
        notify_file_found(state, filename, source_addr, source_port);
        return;
    }

    if hop_count == 0 {
        println!("[Query] Hop count reached 0, dropping query for '{}'", filename);
        return;
    }

    let (successor_ip, successor_port) = {
        let s = state.lock().unwrap();
        if s.is_alone() {
            println!("[Query] Only peer in network, file '{}' not found", filename);
            return;
        }
        (s.successor_ip.clone(), s.successor_port)
    };

    let forward_msg = format!(
        "FILE_QUERY {} {} {} {}",
        hop_count - 1, filename, source_addr, source_port
    );

    println!("[Query] Forwarding '{}' to {}:{} (hop_count={})",
             filename, successor_ip, successor_port, hop_count - 1);

    if let Err(e) = send_message(&successor_ip, successor_port, &forward_msg) {
        eprintln!("[Query] Failed to forward query: {}", e);
    }
}

fn notify_file_found(
    state: Arc<Mutex<PeerState>>,
    filename: &str,
    source_addr: &str,
    source_port: u16,
) {
    let (own_addr, own_port) = {
        let s = state.lock().unwrap();
        (s.own_addr(), s.own_port)
    };

    let msg = format!("FILE_FOUND {} {} {}", filename, own_addr, own_port + 1);
    let source_ip_clean = source_addr.split('%').next().unwrap_or(source_addr);

    println!("[Query] Sending FILE_FOUND to {}:{}: {}", source_ip_clean, source_port, msg);

    if let Err(e) = send_message(source_ip_clean, source_port, &msg) {
        eprintln!("[Query] Failed to send FILE_FOUND: {}", e);
    }
}

pub fn query_file(state: Arc<Mutex<PeerState>>, filename: &str, hop_count: u32) {
    if file_exists(filename) {
        println!("[Query] File '{}' is already in ./own", filename);
        return;
    }

    let (own_addr, own_port, successor_ip, successor_port, is_alone) = {
        let s = state.lock().unwrap();
        (
            s.own_addr(),
            s.own_port,
            s.successor_ip.clone(),
            s.successor_port,
            s.is_alone(),
        )
    };

    if is_alone {
        println!("[Query] Only peer in network, no one to query");
        return;
    }

    let msg = format!("FILE_QUERY {} {} {} {}", hop_count, filename, own_addr, own_port);

    println!("[Query] Sending query for '{}' to successor {}:{}", filename, successor_ip, successor_port);

    if let Err(e) = send_message(&successor_ip, successor_port, &msg) {
        eprintln!("[Query] Failed to send FILE_QUERY: {}", e);
    }
}

pub fn list_own_files() -> Vec<String> {
    list_files()
}