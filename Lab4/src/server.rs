use std::net::UdpSocket;
use crate::decryption::decrypt_round;
use crate::file_io::read_key_file;
use crate::utilities::{BulletinMessage, current_timestamp, serialize_messages};

const MAX_MESSAGES: usize = 5;
const MAX_PACKET_SIZE: usize = 256; // 250 char max message, encrypted 1:1 so 256 is plenty

pub fn run_server(port: u16) -> std::io::Result<()> {
    let bind_addr = format!("0.0.0.0:{}", port);
    let socket = UdpSocket::bind(&bind_addr)?;
    println!("Server listening on {}", bind_addr);

    let key_bytes = read_key_file("key.txt")
        .expect("Server could not read key.txt");

    let mut history: Vec<BulletinMessage> = Vec::new();
    let mut buf = [0u8; MAX_PACKET_SIZE];

    loop {
        let (amt, src) = socket.recv_from(&mut buf)?;
        let sender_ip = src.ip().to_string();
        let encrypted_bytes = &buf[..amt];

        let decrypted = decrypt_payload(encrypted_bytes, &key_bytes);

        let content = match String::from_utf8(decrypted) {
            Ok(s) => s.trim_end_matches('\0').to_string(),
            Err(_) => {
                eprintln!("Could not decode message from {}, skipping", sender_ip);
                let response = serialize_messages(&history);
                socket.send_to(&response, src)?;
                continue;
            }
        };

        println!("\n[New message from {}]", sender_ip);
        println!("  Content: {}", content);

        history.push(BulletinMessage {
            content,
            sender_ip,
            timestamp: current_timestamp(),
        });

        if history.len() > MAX_MESSAGES {
            history.drain(0..history.len() - MAX_MESSAGES);
        }

        print_bulletin_board(&history);

        let response = serialize_messages(&history);
        socket.send_to(&response, src)?;
    }
}

fn decrypt_payload(data: &[u8], key_bytes: &[u8]) -> Vec<u8> {
    let mut decrypted = Vec::new();
    for (i, chunk) in data.chunks(2).enumerate() {
        let block = if chunk.len() == 2 {
            ((chunk[0] as u16) << 8) | (chunk[1] as u16)
        } else {
            (chunk[0] as u16) << 8
        };
        let key = key_bytes[i % 8];
        let dec = decrypt_round(block, key);
        decrypted.push((dec >> 8) as u8);
        decrypted.push((dec & 0xFF) as u8);
    }
    decrypted
}

fn print_bulletin_board(messages: &[BulletinMessage]) {
    println!("\n========== BULLETIN BOARD ==========");
    if messages.is_empty() {
        println!("  (no messages yet)");
    }
    for (i, msg) in messages.iter().enumerate() {
        println!("  [{}] {} | {} | {}", i + 1, msg.timestamp, msg.sender_ip, msg.content);
    }
    println!("====================================\n");
}