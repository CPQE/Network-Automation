use std::net::TcpListener;
use std::io::{self, Read, Write, BufRead};
use std::fs::File;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;
use std::thread;

const CHUNK_SIZE: usize = 1024;
const MAX_CONCURRENT: usize = 5;
const MAX_QUEUED: usize = 5;

pub fn run_tcp_server(port: u16) -> io::Result<()> {
    let bind_addr = format!("[::]:{}", port);
    let listener = TcpListener::bind(&bind_addr)?;
    println!("TCP Server listening on {}", bind_addr);
    println!("Max concurrent transfers: {}", MAX_CONCURRENT);
    println!("Max queued transfers: {}", MAX_QUEUED);

    // Shared active thread counter — decremented when a transfer finishes
    let active_count = Arc::new(Mutex::new(0usize));

    // Shared FIFO queue of waiting connections
    let queue: Arc<Mutex<VecDeque<TcpStream>>> = Arc::new(Mutex::new(VecDeque::new()));

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let peer = stream.peer_addr().unwrap().to_string();
                println!("\n[New connection from {}]", peer);

                let mut active = active_count.lock().unwrap();
                let mut q = queue.lock().unwrap();

                if *active < MAX_CONCURRENT {
                    // Slot available, dispatch immediately
                    *active += 1;
                    drop(active);
                    drop(q);

                    let active_count_clone = Arc::clone(&active_count);
                    let queue_clone = Arc::clone(&queue);

                    thread::spawn(move || {
                        handle_transfer(stream, peer);
                        // Transfer done, decrement active count and check queue
                        drain_queue(active_count_clone, queue_clone);
                    });
                } else if q.len() < MAX_QUEUED {
                    // No slot available but queue has room
                    println!("All slots busy, queuing connection from {} ({}/{})", peer, q.len() + 1, MAX_QUEUED);
                    q.push_back(stream);
                } else {
                    // Both active slots and queue are full, reject immediately
                    println!("Server full, rejecting connection from {}", peer);
                    let mut s = stream;
                    let _ = s.write_all(b"SERVER_FULL\n");
                }
            }
            Err(e) => {
                eprintln!("Connection failed: {}", e);
            }
        }
    }

    Ok(())
}

// Called when a transfer thread finishes. Decrements the active count and
// checks if anything is waiting in the queue to be dispatched.
fn drain_queue(active_count: Arc<Mutex<usize>>, queue: Arc<Mutex<VecDeque<TcpStream>>>) {
    let mut active = active_count.lock().unwrap();
    let mut q = queue.lock().unwrap();

    *active -= 1;

    // If something is waiting in the queue, pop it and spawn a new thread
    if let Some(next_stream) = q.pop_front() {
        let peer = next_stream.peer_addr()
            .map(|a| a.to_string())
            .unwrap_or_else(|_| "unknown".to_string());
        println!("[Dequeuing connection from {}]", peer);
        *active += 1;

        drop(active);
        drop(q);

        let active_count_clone = Arc::clone(&active_count);
        let queue_clone = Arc::clone(&queue);

        thread::spawn(move || {
            handle_transfer(next_stream, peer);
            drain_queue(active_count_clone, queue_clone);
        });
    }
}

// Handles a single file transfer from filename negotiation through to receipt
fn handle_transfer(mut stream: TcpStream, peer: String) {
    println!("[{}] Handling transfer", peer);
    if let Err(e) = handle_client(&mut stream) {
        eprintln!("[{}] Error during transfer: {}", peer, e);
    }
}

// Handles accept/reject negotiation and file receipt for one client
fn handle_client(stream: &mut TcpStream) -> io::Result<()> {
    // Read the filename proposed by the client
    let filename = read_line_from_stream(stream)?;
    println!("Client wants to transfer: '{}'", filename);

    // Prompt server operator to accept or reject
    print!("Accept file? (yes/no): ");
    io::stdout().flush()?;
    let mut response = String::new();
    io::stdin().lock().read_line(&mut response)?;

    if response.trim().eq_ignore_ascii_case("yes") {
        stream.write_all(b"ACCEPT\n")?;

        // Prompt for local save name
        print!("Save file as: ");
        io::stdout().flush()?;
        let mut save_name = String::new();
        io::stdin().lock().read_line(&mut save_name)?;
        let save_name = save_name.trim().to_string();

        receive_file(stream, &save_name)?;

        stream.write_all(b"SUCCESS\n")?;
        println!("File saved as '{}'", save_name);
    } else {
        stream.write_all(b"REJECT\n")?;
        println!("Rejected.");
    }

    Ok(())
}

// Receives 1024 byte chunks from the stream and writes them to disk until stream closes
fn receive_file(stream: &mut TcpStream, save_path: &str) -> io::Result<()> {
    let mut file = File::create(save_path)?;
    let mut buf = [0u8; CHUNK_SIZE];
    let mut total = 0;

    loop {
        let n = stream.read(&mut buf)?;
        if n == 0 {
            break;
        }
        file.write_all(&buf[..n])?;
        total += n;
        println!("  Received {} bytes ({} total)", n, total);
    }

    println!("Transfer complete: {} bytes total", total);
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