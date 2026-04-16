mod bootstrap;
mod peer;
mod listener;
mod query;
mod transfer;
mod files;
mod cli;

use std::sync::{Arc, Mutex};
use std::thread;
use peer::PeerState;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 6 {
        eprintln!("Usage: peer <own_ip> <interface> <own_port> <bootstrap_ip> <bootstrap_port>");
        std::process::exit(1);
    }

    let own_ip          = args[1].clone();
    let interface       = args[2].clone();
    let own_port: u16   = args[3].parse().expect("Invalid own port");
    let bootstrap_ip    = args[4].clone();
    let bootstrap_port: u16 = args[5].parse().expect("Invalid bootstrap port");

    let own_addr = format!("{}%{}", own_ip, interface);

    // Ensure ./own directory exists before anything else
    files::init_own_dir();

    println!("Starting peer {} on port {}", own_addr, own_port);

    let state = Arc::new(Mutex::new(PeerState::new(
        own_ip.clone(),
        interface.clone(),
        own_port,
    )));

    match bootstrap::register(&own_addr, own_port, &bootstrap_ip, bootstrap_port, &own_ip, &interface) {
        Ok(Some((existing_ip, existing_port))) => {
            println!("Got existing peer {}:{}, joining ring...", existing_ip, existing_port);
            peer::join_ring(Arc::clone(&state), &existing_ip, existing_port)
                .expect("Failed to join ring");
        }
        Ok(None) => {
            println!("First peer in network, pointing successor/predecessor to self");
            let mut s = state.lock().unwrap();
            s.successor_ip = own_ip.clone();
            s.successor_interface = interface.clone();
            s.successor_port = own_port;
            s.predecessor_ip = own_ip.clone();
            s.predecessor_interface = interface.clone();
            s.predecessor_port = own_port;
        }
        Err(e) => {
            eprintln!("Bootstrap registration failed: {}", e);
            std::process::exit(1);
        }
    }

    {
        let s = state.lock().unwrap();
        println!("Joined network successfully");
        s.print_state();
    }

    // Spawn UDP listener thread
    let state_listener = Arc::clone(&state);
    let own_ip_listener = own_ip.clone();
    let interface_listener = interface.clone();
    thread::spawn(move || {
        listener::run_listener(state_listener, &own_ip_listener, &interface_listener, own_port);
    });

    // Spawn TCP file server thread on own_port + 1
    let state_transfer = Arc::clone(&state);
    let own_ip_transfer = own_ip.clone();
    thread::spawn(move || {
        transfer::run_file_server(state_transfer, &own_ip_transfer, own_port + 1);
    });

    // Run CLI on main thread
    println!("Starting CLI...");
    cli::run_cli(
        Arc::clone(&state),
        &own_addr,
        own_port,
        &bootstrap_ip,
        bootstrap_port,
    );
}