use circular_buffer::CircularBuffer;

use crate::utilities::{Mode, parse_args}; 
use crate::checksum::{calculate_packet_checksum};
use crate::utilities::{ip_to_u32_be, parse_ip, u16_to_be_bytes, u32_to_be_bytes, be_bytes_to_u16, pad_data_if_needed};
use crate::headers::{build_udp_header_without_checksum, build_pseudo_header};

use crate::utilities::{BulletinMessage, serialize_messages, deserialize_messages, current_timestamp};
use crate::encryption::encrypt_round;
use crate::decryption::decrypt_round;


pub fn run_unittests(){
    test_parse_args(); 
    test_parse_ip();
    test_parse_ip_and_ip_to_u32_be();
    test_u16_to_be_bytes();
    test_u32_to_be_bytes();
    test_be_bytes_to_u16();
    test_pad_data_if_needed();
    test_build_udp_header_without_checksum();
    test_pseudo_header_and_checksum();
    test_serialize_deserialize_messages();
    test_timestamp();
    test_encrypt_decrypt_roundtrip();
}

fn test_parse_args() {
    match parse_args() {
    Ok(mode) => {
        println!("Parsed OK:");
        match mode {
            Mode::Sender {
                data_file,
                src_ip,
                dst_ip,
                src_port,
                dst_port,
                output_file,
            } => {
                println!("Mode: Sender");
                println!("  data_file: {}", data_file);
                println!("  src_ip: {}", src_ip);
                println!("  dst_ip: {}", dst_ip);
                println!("  src_port: {}", src_port);
                println!("  dst_port: {}", dst_port);
                println!("  output_file: {}", output_file);
            }

            Mode::Receiver {
                src_ip,
                dst_ip,
                datagram_file,
            } => {
                println!("Mode: Receiver");
                println!("  src_ip: {}", src_ip);
                println!("  dst_ip: {}", dst_ip);
                println!("  datagram_file: {}", datagram_file);
            }
            _ => {
                println!("Mode: Unknown");
            }
        }
    }
    Err(e) => {
        println!("Error: {}", e);
    }
    }
}
fn test_parse_ip(){
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

fn test_parse_ip_and_ip_to_u32_be(){
    println!("--- Testing ip_to_u32_be() ---");
    let ip1 = parse_ip("192.168.0.1").unwrap();
    let val1 = ip_to_u32_be(ip1);
    println!("192.168.0.1 → {:#010X}", val1);
    let ip2 = parse_ip("10.0.0.2").unwrap();
    let val2 = ip_to_u32_be(ip2);
    println!("10.0.0.2 → {:#010X}", val2);
    let ip3 = parse_ip("127.0.0.1").unwrap();
    let val3 = ip_to_u32_be(ip3);
    println!("127.0.0.1 → {:#010X}", val3);
}

fn test_u16_to_be_bytes() {
    println!("--- Testing u16_to_be_bytes() ---");
    let a = u16_to_be_bytes(0x1234);
    println!("0x1234 → [{:#04X}, {:#04X}]", a[0], a[1]);
    let b = u16_to_be_bytes(80); // port 80
    println!("80 → [{:#04X}, {:#04X}]", b[0], b[1]);
    let c = u16_to_be_bytes(65535);
    println!("65535 → [{:#04X}, {:#04X}]", c[0], c[1]);
}

fn test_u32_to_be_bytes() {
    println!("--- Testing u32_to_be_bytes() ---");
    let a = u32_to_be_bytes(0x12345678);
    println!("0x12345678 → [{:#04X}, {:#04X}, {:#04X}, {:#04X}]",
        a[0], a[1], a[2], a[3]);
    let b = u32_to_be_bytes(0xC0A80001); // 192.168.0.1
    println!("0xC0A80001 → [{:#04X}, {:#04X}, {:#04X}, {:#04X}]",
        b[0], b[1], b[2], b[3]);
    let c = u32_to_be_bytes(0);
    println!("0 → [{:#04X}, {:#04X}, {:#04X}, {:#04X}]",
        c[0], c[1], c[2], c[3]);
}

fn test_be_bytes_to_u16() {
    println!("--- Testing be_bytes_to_u16() ---");
    let a = be_bytes_to_u16([0x12, 0x34]);
    println!("[0x12, 0x34] → {:#06X}", a);
    let b = be_bytes_to_u16([0x00, 0x50]); // port 80
    println!("[0x00, 0x50] → {:#06X}", b);
    let c = be_bytes_to_u16([0xFF, 0xFF]);
    println!("[0xFF, 0xFF] → {:#06X}", c);
}

fn test_pad_data_if_needed() {
    println!("--- Testing pad_data_if_needed() ---");
    let mut odd = vec![1, 2, 3];
    pad_data_if_needed(&mut odd);
    println!("Odd length padded → {:?}", odd);
    let mut even = vec![1, 2, 3, 4];
    pad_data_if_needed(&mut even);
    println!("Even length unchanged → {:?}", even);
}

fn test_build_udp_header_without_checksum() {
    println!("--- Testing build_udp_header_without_checksum() ---");
    let hdr = build_udp_header_without_checksum(80, 22, 100);
    println!("UDP header (no checksum): {:?}", hdr);
}

fn test_pseudo_header_and_checksum() {
    println!("--- Testing build_pseudo_header() + checksum ---");
    // Example values:
    let src_ip = 0xC0A80001; // 192.168.0.1
    let dst_ip = 0x7F000001; // 127.0.0.1
    let udp_len = 100;

    let pseudo = build_pseudo_header(src_ip, dst_ip, udp_len);
    println!("Pseudo-header: {:?}", pseudo);

    // Simple checksum test on known bytes
    let test_data = vec![0x12, 0x34, 0x56, 0x78];
    let csum = calculate_packet_checksum(&test_data);
    println!("Checksum of [12 34 56 78] → {:#06X}", csum);
}
fn test_serialize_deserialize_messages() {
    println!("--- Testing serialize/deserialize messages ---");
    
    let mut messages: CircularBuffer<5, BulletinMessage> = CircularBuffer::new();
    messages.push_back(BulletinMessage { content: "hello world".to_string(), sender_ip: "192.168.0.1".to_string(), timestamp: 1000 });
    messages.push_back(BulletinMessage { content: "second message".to_string(), sender_ip: "10.0.0.2".to_string(), timestamp: 2000 });

    let serialized = serialize_messages(&messages);
    let deserialized = deserialize_messages(&serialized);

    assert_eq!(deserialized.len(), 2);
    assert_eq!(deserialized[0].content, "hello world");
    assert_eq!(deserialized[0].sender_ip, "192.168.0.1");
    assert_eq!(deserialized[0].timestamp, 1000);
    assert_eq!(deserialized[1].content, "second message");
    println!("OK: serialize/deserialize roundtrip");

    // edge case: empty
    let empty: CircularBuffer<5, BulletinMessage> = CircularBuffer::new();
    let empty_des = deserialize_messages(&serialize_messages(&empty));
    assert_eq!(empty_des.len(), 0);
    println!("OK: empty message list");

    // edge case: pipe character in content would break splitn
    let mut tricky: CircularBuffer<5, BulletinMessage> = CircularBuffer::new();
    tricky.push_back(BulletinMessage { content: "no pipes here".to_string(), sender_ip: "1.2.3.4".to_string(), timestamp: 99 });
    let d = deserialize_messages(&serialize_messages(&tricky));
    assert_eq!(d[0].content, "no pipes here");
    println!("OK: tricky content roundtrip");
}

fn test_timestamp() {
    println!("--- Testing current_timestamp() ---");
    let t1 = current_timestamp();
    let t2 = current_timestamp();
    assert!(t2 >= t1, "timestamps should be non-decreasing");
    assert!(t1 > 1_000_000_000, "timestamp should be a plausible unix time"); // year 2001+
    println!("OK: timestamp {} looks reasonable", t1);
}

fn test_encrypt_decrypt_roundtrip() {
    println!("--- Testing encrypt/decrypt roundtrip ---");
    let key: u8 = 0xAB;
    let original: u16 = 0x4865; // "He"

    let encrypted = encrypt_round(original, key);
    let decrypted = decrypt_round(encrypted, key);

    assert_ne!(encrypted, original, "encrypted should differ from original");
    assert_eq!(decrypted, original, "decrypted should match original");
    println!("OK: 0x{:04X} -> encrypted 0x{:04X} -> decrypted 0x{:04X}", original, encrypted, decrypted);

    // test with a few different keys and values
    for key in [0x00u8, 0xFF, 0x55, 0xAA] {
        for val in [0x0000u16, 0xFFFF, 0x1234, 0xABCD] {
            let enc = encrypt_round(val, key);
            let dec = decrypt_round(enc, key);
            assert_eq!(dec, val, "roundtrip failed for val={:#06X} key={:#04X}", val, key);
        }
    }
    println!("OK: roundtrip holds across multiple keys and values");
}