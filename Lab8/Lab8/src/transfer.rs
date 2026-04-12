use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::io::{self, Read, Write};
use std::fs;
use std::thread;
use crate::peer::{PeerState};
use crate::files::{own_file_path, file_exists};

const CHUNK_SIZE: usize = 1024;

pub fn run_file_server(
    state: Arc<Mutex<PeerState>>,
    own_ip: &str,
    port: u16,
) {
    let bind_addr = format!("[{}]:{}", own_ip, port);
    let listener = TcpListener::bind(&bind_addr)
        .expect("Failed to bind file server");

    println!("[Transfer] File server listening on {}", bind_addr);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let state_clone = Arc::clone(&state);
                thread::spawn(move || {
                    if let Err(e) = serve_file(stream, state_clone) {
                        eprintln!("[Transfer] Error serving file: {}", e);
                    }
                });
            }
            Err(e) => eprintln!("[Transfer] Accept error: {}", e),
        }
    }
}

fn serve_file(mut stream: TcpStream, _state: Arc<Mutex<PeerState>>) -> io::Result<()> {
    let peer = stream.peer_addr()?.to_string();
    println!("[Transfer] Incoming request from {}", peer);

    let filename = read_line(&mut stream)?;
    println!("[Transfer] Requested: '{}'", filename);

    if !file_exists(&filename) {
        stream.write_all(b"ERROR File not found\n")?;
        return Ok(());
    }

    stream.write_all(b"OK\n")?;

    let mut file = fs::File::open(own_file_path(&filename))?;
    let mut buf = [0u8; CHUNK_SIZE];
    let mut total = 0;

    loop {
        let n = file.read(&mut buf)?;
        if n == 0 { break; }
        stream.write_all(&buf[..n])?;
        total += n;
    }

    stream.shutdown(std::net::Shutdown::Write)?;
    println!("[Transfer] Sent '{}' ({} bytes) to {}", filename, total, peer);
    Ok(())
}

pub fn handle_file_found(_state: Arc<Mutex<PeerState>>, parts: &[&str]) {
    if parts.len() < 4 {
        eprintln!("[Transfer] Malformed FILE_FOUND: {:?}", parts);
        return;
    }

    let filename  = parts[1].to_string();
    let peer_addr = parts[2];
    let peer_port: u16 = match parts[3].parse() {
        Ok(p) => p,
        Err(_) => {
            eprintln!("[Transfer] Invalid port in FILE_FOUND");
            return;
        }
    };

    let peer_ip_clean = peer_addr.split('%').next().unwrap_or(peer_addr).to_string();

    println!("[Transfer] FILE_FOUND '{}' at {}:{}, downloading...",
             filename, peer_ip_clean, peer_port);

    thread::spawn(move || {
        if let Err(e) = download_file(&peer_ip_clean, peer_port, &filename) {
            eprintln!("[Transfer] Download failed: {}", e);
        }
    });
}

fn download_file(peer_ip: &str, peer_port: u16, filename: &str) -> io::Result<()> {
    let peer_addr = format!("[{}]:{}", peer_ip, peer_port);
    println!("[Transfer] Connecting to {} for '{}'", peer_addr, filename);

    let mut stream = TcpStream::connect(&peer_addr)?;
    stream.write_all(format!("{}\n", filename).as_bytes())?;

    let response = read_line(&mut stream)?;
    if response != "OK" {
        eprintln!("[Transfer] Server rejected: {}", response);
        return Ok(());
    }

    let filepath = own_file_path(filename);
    let mut file = fs::File::create(&filepath)?;
    let mut buf = [0u8; CHUNK_SIZE];
    let mut total = 0;

    loop {
        let n = stream.read(&mut buf)?;
        if n == 0 { break; }
        file.write_all(&buf[..n])?;
        total += n;
    }

    println!("[Transfer] Downloaded '{}' ({} bytes) to {}", filename, total, filepath);
    Ok(())
}

fn read_line(stream: &mut TcpStream) -> io::Result<String> {
    let mut result = String::new();
    let mut buf = [0u8; 1];
    loop {
        let n = stream.read(&mut buf)?;
        if n == 0 || buf[0] == b'\n' { break; }
        result.push(buf[0] as char);
    }
    Ok(result.trim().to_string())
}