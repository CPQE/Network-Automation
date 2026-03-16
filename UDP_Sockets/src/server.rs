use std::net::UdpSocket;
use circular_buffer::CircularBuffer;
use crate::decryption::decrypt_round;
use crate::file_io::read_key_file;
use crate::utilities::{BulletinMessage, current_timestamp, format_timestamp, serialize_messages};

const MAX_PACKET_SIZE: usize = 256;
//bind to socket
pub fn run_server(port: u16) -> std::io::Result<()> {
    let bind_addr = format!("[::]:{}", port); //:: lets any address connect (ipv6)
    let socket = UdpSocket::bind(&bind_addr)?;
    println!("Server listening on {}", bind_addr);
    //read in key file
    let key_bytes = read_key_file("key.txt") 
        .expect("Server could not read key.txt");

    // CircularBuffer with capacity 5. When a 6th message arrives, the oldest
    // is automatically evicted. No manual length check needed.
    let mut history: CircularBuffer<5, BulletinMessage> = CircularBuffer::new();
    let mut buf = [0u8; MAX_PACKET_SIZE];

    loop { //server runs in infinite loop until user presses ctrl+c to terminate
        // Block until a packet arrives, then record sender address and payload size
        let (amt, src) = socket.recv_from(&mut buf)?;
        let sender_ip = src.ip().to_string();
        let encrypted_bytes = &buf[..amt];

        // Decrypt the incoming payload using the shared key file
        let decrypted = decrypt_payload(encrypted_bytes, &key_bytes); //decrypt received payload

        // Attempt to decode the decrypted bytes as UTF-8 text.
        // On failure, still respond so the client does not hang waiting.
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

        // Push new message to the back. If history is already at capacity (5),
        // the oldest message at the front is automatically dropped.
        history.push_back(BulletinMessage {
            content,
            sender_ip,
            timestamp: current_timestamp(),
        });

        // Print the current state of the bulletin board to stdout
        print_bulletin_board(&history);

        // Serialize the current history and send it back to the client
        let response = serialize_messages(&history);
        socket.send_to(&response, src)?;
    }
}

// Decrypts a byte slice using the same 16-bit block scheme as the encryption step.
// Each pair of bytes is treated as a 16-bit block and decrypted with the
// corresponding key byte, cycling through the 8-byte key file as needed.
// If the input has an odd number of bytes, the final byte is treated as
// the high byte of a block with a zero low byte.
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

// Prints the current bulletin board state to stdout.
// Shows up to 5 entries with their index, timestamp,
// sender IP, and decrypted message content.
fn print_bulletin_board(messages: &CircularBuffer<5, BulletinMessage>) {
    println!("\n========== BULLETIN BOARD ==========");
    if messages.is_empty() {
        println!("  (no messages yet)");
    }
    for (i, msg) in messages.iter().enumerate() {
        println!("  [{}] {} | {} | {}", i + 1, format_timestamp(msg.timestamp), msg.sender_ip, msg.content);
    }
    println!("====================================\n");
}