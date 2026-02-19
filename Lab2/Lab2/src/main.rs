
mod encryption;
mod decryption; 
mod utilities; 
mod checksum; 
mod headers;
mod file_io;
mod sender;
mod receiver; 
mod unittests;

use crate::utilities::{Mode, parse_args}; 
use crate::checksum::{calculate_packet_checksum};
use crate::utilities::{ip_to_u32_be, parse_ip, u16_to_be_bytes, u32_to_be_bytes, be_bytes_to_u16, pad_data_if_needed};
use crate::headers::{build_udp_header_without_checksum, build_pseudo_header};
use crate::unittests::run_unittests;
use crate::file_io::{read_file_bytes, read_key_file};
use crate::sender::{build_udp_datagram, write_datagram_to_file};
use crate::receiver::{parse_udp_datagram, write_payload_to_file};

fn main() {
    //to test sender: cargo run -- name.txt 192.168.0.1 124.26.12.24 80 22 datagram
    //to test receiver: cargo run -- 192.168.0.1 124.26.12.24 datagram
    run_unittests(); 
    match parse_args() {
        // -------------------------
        // SENDER MODE
        // -------------------------
        Ok(Mode::Sender {
            data_file,
            src_ip,
            dst_ip,
            src_port,
            dst_port,
            output_file,
        }) => {
            println!("Running in SENDER mode");
            // Convert IP strings to u32
            let src_ip_u32 = utilities::ip_to_u32_be(
                utilities::parse_ip(&src_ip).expect("Invalid source IP")
            );
            let dst_ip_u32 = utilities::ip_to_u32_be(
                utilities::parse_ip(&dst_ip).expect("Invalid destination IP")
            );
            // Build datagram
            let datagram = sender::build_udp_datagram(
                &data_file,
                src_ip_u32,
                dst_ip_u32,
                src_port,
                dst_port,
            ).expect("Failed to build datagram");

            // Write datagram to file
            sender::write_datagram_to_file(&output_file, &datagram)
                .expect("Failed to write datagram");
            println!("Sender: wrote {}", output_file);
        }
        // -------------------------
        // RECEIVER MODE
        // -------------------------
        Ok(Mode::Receiver {
            src_ip,
            dst_ip,
            datagram_file,
        }) => {
            println!("Running in RECEIVER mode");

            // Convert IP strings to u32
            let src_ip_u32 = utilities::ip_to_u32_be(
                utilities::parse_ip(&src_ip).expect("Invalid source IP")
            );
            let dst_ip_u32 = utilities::ip_to_u32_be(
                utilities::parse_ip(&dst_ip).expect("Invalid destination IP")
            );

            // Parse + decrypt datagram
            let parsed = receiver::parse_udp_datagram(
                &datagram_file,
                src_ip_u32,
                dst_ip_u32,
            ).expect("Failed to parse datagram");

            // Write recovered payload
            receiver::write_payload_to_file("recovered.txt", &parsed.payload)
                .expect("Failed to write recovered payload");

            println!("Receiver: wrote recovered.txt");
        }
        // -------------------------
        // ERROR HANDLING
        // -------------------------
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }

}