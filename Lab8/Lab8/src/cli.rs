use std::sync::{Arc, Mutex};
use std::io::{self, BufRead, Write};
use crate::peer::{PeerState, leave_ring};
use crate::query::{query_file, list_own_files};
use crate::bootstrap::unregister;

// Runs the user prompt loop on the main thread.
// Blocks until user types 'leave'.
pub fn run_cli(
    state: Arc<Mutex<PeerState>>,
    own_addr: &str,
    own_port: u16,
    bootstrap_ip: &str,
    bootstrap_port: u16,
) {
    let stdin = io::stdin();
    let own_addr = own_addr.to_string();
    let bootstrap_ip = bootstrap_ip.to_string();

    println!("\nPeer ready. Commands: query <hop_count> <filename> | display | leave\n");

    loop {
        print!("> ");
        io::stdout().flush().unwrap();

        let mut line = String::new();
        match stdin.lock().read_line(&mut line) {
            Ok(0) => {
                // EOF — treat as leave
                println!("EOF received, leaving...");
                break;
            }
            Ok(_) => {}
            Err(e) => {
                eprintln!("Failed to read input: {}", e);
                break;
            }
        }

        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.splitn(3, ' ').collect();
        match parts[0] {
            "query" => {
                if parts.len() < 3 {
                    println!("Usage: query <hop_count> <filename>");
                    continue;
                }
                let hop_count: u32 = match parts[1].parse() {
                    Ok(h) => h,
                    Err(_) => {
                        println!("Invalid hop count, must be a number");
                        continue;
                    }
                };
                let filename = parts[2];
                query_file(Arc::clone(&state), filename, hop_count);
            }

            "display" => {
                display(&state);
            }

            "leave" => {
                println!("Leaving network...");
                // Notify successor and predecessor
                if let Err(e) = leave_ring(Arc::clone(&state)) {
                    eprintln!("Error notifying peers: {}", e);
                }
                // Unregister from bootstrap
                if let Err(e) = unregister(&own_addr, own_port, &bootstrap_ip, bootstrap_port) {
                    eprintln!("Error unregistering from bootstrap: {}", e);
                }
                println!("Goodbye.");
                std::process::exit(0);
            }

            _ => {
                println!("Unknown command '{}'. Commands: query <hop_count> <filename> | display | leave", parts[0]);
            }
        }
    }
}

// Displays own IP/port, successor, predecessor, and files in ./own
fn display(state: &Arc<Mutex<PeerState>>) {
    let s = state.lock().unwrap();
    s.print_state();

    let files = list_own_files();
    println!("Files in ./own ({} total):", files.len());
    if files.is_empty() {
        println!("  (none)");
    } else {
        for f in &files {
            println!("  {}", f);
        }
    }
    println!();
}