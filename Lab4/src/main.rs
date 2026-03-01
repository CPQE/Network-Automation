mod encryption;
mod decryption; 
mod utilities; 
mod checksum; 
mod headers;
mod file_io;
mod sender;
mod receiver; 
mod unittests;
mod server;
mod client;

use crate::utilities::{Mode, parse_args};
use crate::utilities::{ip_to_u32_be, parse_ip};
use crate::unittests::run_unittests;
use crate::sender::{build_udp_datagram, write_datagram_to_file};
use crate::receiver::{parse_udp_datagram, write_payload_to_file};

fn main() {

    //assignment 3: 
    // cargo run -- server 8080
    // cargo run -- client 127.0.0.1 8080 (stdin mode)
    // cargo run -- client 127.0.0.1 8080 messages.txt (file mode)
    //assignment 2:
    //to test sender: cargo run -- name.txt 192.168.0.1 124.26.12.24 80 22 datagram
    //to test receiver: cargo run -- 192.168.0.1 124.26.12.24 datagram
    let testing = false; 
    if testing {
        run_unittests(); 
    }

    match parse_args() {
        Ok(Mode::Sender { data_file, src_ip, dst_ip, src_port, dst_port, output_file }) => {
            println!("Running in SENDER mode");
            let src_ip_u32 = ip_to_u32_be(parse_ip(&src_ip).expect("Invalid source IP"));
            let dst_ip_u32 = ip_to_u32_be(parse_ip(&dst_ip).expect("Invalid destination IP"));
            let datagram = build_udp_datagram(&data_file, src_ip_u32, dst_ip_u32, src_port, dst_port)
                .expect("Failed to build datagram");
            write_datagram_to_file(&output_file, &datagram)
                .expect("Failed to write datagram");
            println!("Sender: wrote {}", output_file);
        }

        Ok(Mode::Receiver { src_ip, dst_ip, datagram_file }) => {
            println!("Running in RECEIVER mode");
            let src_ip_u32 = ip_to_u32_be(parse_ip(&src_ip).expect("Invalid source IP"));
            let dst_ip_u32 = ip_to_u32_be(parse_ip(&dst_ip).expect("Invalid destination IP"));
            let parsed = parse_udp_datagram(&datagram_file, src_ip_u32, dst_ip_u32)
                .expect("Failed to parse datagram");
            write_payload_to_file("recovered.txt", &parsed.payload)
                .expect("Failed to write recovered payload");
            println!("Receiver: wrote recovered.txt");
        }

        Ok(Mode::Server { port }) => {
            println!("Running in SERVER mode");
            server::run_server(port)
                .expect("Server error");
        }

        Ok(Mode::Client { server_ip, port, input_file }) => {
            println!("Running in CLIENT mode");
            client::run_client(&server_ip, port, input_file.as_deref())
                .expect("Client error");
        }

        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}