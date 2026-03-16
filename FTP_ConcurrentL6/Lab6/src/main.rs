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
mod tcp_server;
mod tcp_client;

use crate::utilities::{Mode, parse_args};
use crate::utilities::{ip_to_u32_be, parse_ip};
use crate::unittests::run_unittests;
use crate::sender::{build_udp_datagram, write_datagram_to_file};
use crate::receiver::{parse_udp_datagram, write_payload_to_file};

fn main() {
    // Lab 2/3 (manual UDP datagram builder):
    // cargo run -- sender <data_file> <src_ip> <dst_ip> <src_port> <dst_port> <output_file>
    // cargo run -- receiver <src_ip> <dst_ip> <datagram_file>

    // Lab 4 (UDP encrypted bulletin board):
    // cargo run -- server 8080
    // cargo run -- client ::1 8080           (stdin mode, IPv6 loopback)
    // cargo run -- client ::1 8080 name.txt  (file mode, IPv6 loopback)

    // Lab 5 (TCP file transfer):
    // cargo run -- tcp_server 8080
    // cargo run -- tcp_client ::1 8080 name.txt

    // Set to true to run unit tests before dispatching to a mode
    let testing = false; 
    if testing {
        run_unittests(); 
    }

    match parse_args() {
        // Lab 2/3: manually builds a UDP datagram with checksum and writes it to a file
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

        // Lab 2/3: reads a datagram file, verifies checksum, decrypts payload, writes recovered file
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

        // Lab 4: starts the UDP bulletin board server, listens for encrypted messages
        Ok(Mode::Server { port }) => {
            println!("Running in SERVER mode");
            server::run_server(port)
                .expect("Server error");
        }

        // Lab 4: encrypts a message and sends it to the bulletin board server via UDP
        Ok(Mode::Client { server_ip, port, input_file }) => {
            println!("Running in CLIENT mode");
            client::run_client(&server_ip, port, input_file.as_deref())
                .expect("Client error");
        }

        // Lab 5: starts the TCP file transfer server, waits for incoming connections
        Ok(Mode::TcpServer { port }) => {
            println!("Running in TCP SERVER mode");
            tcp_server::run_tcp_server(port)
                .expect("TCP Server error");
        }

        // Lab 5: connects to the TCP server and transfers the specified file in 1024 byte chunks
        Ok(Mode::TcpClient { server_ip, port, filename }) => {
            println!("Running in TCP CLIENT mode");
            tcp_client::run_tcp_client(&server_ip, port, &filename)
                .expect("TCP Client error");
        }

        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}