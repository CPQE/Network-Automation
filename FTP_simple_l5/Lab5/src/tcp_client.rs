use std::net::TcpStream;
use std::io::{self, Read, Write};
use std::fs::File;

const CHUNK_SIZE: usize = 1024;

pub fn run_tcp_client(server_ip: &str, port: u16, filename: &str) -> io::Result<()> {
    let server_addr = format!("[{}]:{}", server_ip, port);
    println!("Connecting to {}...", server_addr);

    let mut stream = TcpStream::connect(&server_addr)?;
    println!("Connected.");

    // Send filename to server
    stream.write_all(format!("{}\n", filename).as_bytes())?;
    println!("Sent filename: '{}'", filename);

    // Wait for server accept/reject response
    let response = read_line_from_stream(&mut stream)?;
    match response.as_str() {
        "ACCEPT" => {
            println!("Server accepted the file. Sending...");
            send_file(&mut stream, filename)?;

            // Wait for server confirmation
            let confirmation = read_line_from_stream(&mut stream)?;
            if confirmation == "SUCCESS" {
                println!("File transfer successful.");
            } else {
                eprintln!("Unexpected server response: {}", confirmation);
            }
        }
        "REJECT" => {
            println!("Server rejected the file. Closing connection.");
        }
        _ => {
            eprintln!("Unknown server response: {}", response);
        }
    }

    Ok(())
}

// Reads the file and sends it in CHUNK_SIZE byte chunks
fn send_file(stream: &mut TcpStream, path: &str) -> io::Result<()> {
    let mut file = File::open(path)
        .map_err(|e| io::Error::new(io::ErrorKind::NotFound, format!("Could not open '{}': {}", path, e)))?;

    let mut buf = [0u8; CHUNK_SIZE];
    let mut total = 0;

    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            // EOF reached, done sending
            break;
        }
        stream.write_all(&buf[..n])?;
        total += n;
        println!("  Sent {} bytes ({} total)", n, total);
    }

    // Shut down the write side to signal EOF to the server
    stream.shutdown(std::net::Shutdown::Write)?;
    println!("Transfer complete: {} bytes sent", total);
    Ok(())
}

// Reads a newline terminated string from the stream byte by byte
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