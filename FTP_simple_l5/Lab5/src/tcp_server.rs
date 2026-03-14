use std::net::{TcpListener, TcpStream};
use std::io::{self, Read, Write, BufRead};
use std::thread;
use std::fs::File;

const CHUNK_SIZE: usize = 1024;

pub fn run_tcp_server(port: u16) -> io::Result<()> {
    let bind_addr = format!("[::]:{}", port);
    let listener = TcpListener::bind(&bind_addr)?;
    println!("TCP Server listening on {}", bind_addr);

    // Accept connections in a loop, spawning a thread for each client
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let peer = stream.peer_addr().unwrap().to_string();
                println!("\n[New connection from {}]", peer);
                thread::spawn(move || {
                    if let Err(e) = handle_client(stream) {
                        eprintln!("Error handling client {}: {}", peer, e);
                    }
                });
            }
            Err(e) => {
                eprintln!("Connection failed: {}", e);
            }
        }
    }

    Ok(())
}

// Handles a single client connection from filename negotiation through to file receipt
fn handle_client(mut stream: TcpStream) -> io::Result<()> {
    // Read the filename sent by the client
    let filename = read_line_from_stream(&mut stream)?;
    println!("Client wants to transfer file: '{}'", filename);

    // Prompt server user to accept or reject
    print!("Accept file? (yes/no): ");
    io::stdout().flush()?;
    let mut response = String::new();
    io::stdin().lock().read_line(&mut response)?;
    let response = response.trim();

    if response.eq_ignore_ascii_case("yes") {
        // Tell client we accept
        stream.write_all(b"ACCEPT\n")?;
        println!("Accepted. Waiting for file data...");

        // Prompt for local save name
        print!("Save file as: ");
        io::stdout().flush()?;
        let mut save_name = String::new();
        io::stdin().lock().read_line(&mut save_name)?;
        let save_name = save_name.trim().to_string();

        // Receive file in chunks and write to disk
        receive_file(&mut stream, &save_name)?;
        println!("File saved as '{}'", save_name);

        // Confirm receipt to client
        stream.write_all(b"SUCCESS\n")?;
    } else {
        // Tell client we reject
        stream.write_all(b"REJECT\n")?;
        println!("Rejected file from client.");
    }

    Ok(())
}

// Receives 1024 byte chunks from the stream and writes them to a file until the stream closes
fn receive_file(stream: &mut TcpStream, save_path: &str) -> io::Result<()> {
    let mut file = File::create(save_path)?;
    let mut buf = [0u8; CHUNK_SIZE];
    let mut total_bytes = 0;

    loop {
        let n = stream.read(&mut buf)?;
        if n == 0 {
            // Stream closed, transfer complete
            break;
        }
        file.write_all(&buf[..n])?;
        total_bytes += n;
        println!("  Received {} bytes ({} total)", n, total_bytes);
    }

    println!("Transfer complete: {} bytes received", total_bytes);
    Ok(())
}

// Reads a newline terminated string from a TCP stream
fn read_line_from_stream(stream: &mut TcpStream) -> io::Result<String> {
    let mut result = String::new();
    let mut buf = [0u8; 1];
    loop {
        let n = stream.read(&mut buf)?;
        if n == 0 || buf[0] == b'\n' {
            break;
        }
        result.push(buf[0] as char);
    }
    Ok(result.trim().to_string())
}