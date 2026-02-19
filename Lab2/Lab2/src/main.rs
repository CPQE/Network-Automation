
mod encryption;
mod decryption; 
mod utilities; 
mod addition; 
// use std::env; 

use crate::utilities::parse_args; 
use crate::utilities::Args;
use crate::addition::calculate_packet_checksum;
use crate::utilities::parse_ip;


fn main() {
    //to test: cargo run -- name.txt 192.168.0.1 124.26.12.24 80 22 datagram
    match parse_args() { //error check parsed command line arguments
        Ok(args) => {
            println!("Parsed OK:\n{:#?}", args);
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }
     println!("--- Testing parse_ip() ---");

    // Test 1: Valid IP
    match parse_ip("192.168.0.1") {
        Ok(bytes) => println!("OK (valid): {:?}", bytes),
        Err(e) => println!("FAILED (valid): {}", e),
    }

    // Test 2: Invalid octet
    match parse_ip("300.1.2.3") {
        Ok(bytes) => println!("FAILED (invalid octet): {:?}", bytes),
        Err(e) => println!("OK (invalid octet): {}", e),
    }

    // Test 3: Not enough octets
    match parse_ip("1.2.3") {
        Ok(bytes) => println!("FAILED (too few octets): {:?}", bytes),
        Err(e) => println!("OK (too few octets): {}", e),
    }

    // Test 4: Too many octets
    match parse_ip("1.2.3.4.5") {
        Ok(bytes) => println!("FAILED (too many octets): {:?}", bytes),
        Err(e) => println!("OK (too many octets): {}", e),
    }

    // Test 5: Non-numeric
    match parse_ip("abc.def.ghi.jkl") {
        Ok(bytes) => println!("FAILED (non-numeric): {:?}", bytes),
        Err(e) => println!("OK (non-numeric): {}", e),
    }

}

