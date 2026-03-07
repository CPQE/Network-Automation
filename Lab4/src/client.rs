use std::net::UdpSocket;
use std::io::{self, BufRead};
use crate::encryption::encrypt_round;
use crate::file_io::{read_key_file, read_file_bytes};
use crate::utilities::{BulletinMessage, deserialize_messages, format_timestamp};

// Response from server can be up to 5 messages so needs more headroom than what the client sends
const MAX_PACKET_SIZE: usize = 512;
const TIMEOUT_SECS: u64 = 5;

// Initializes the UDP socket, loads the key file, and dispatches to file or stdin input mode.
pub fn run_client(server_ip: &str, port: u16, input_file: Option<&str>) -> std::io::Result<()> {
    let socket = UdpSocket::bind("[::]:0")?;
    socket.set_read_timeout(Some(std::time::Duration::from_secs(TIMEOUT_SECS)))?;

    let server_addr = format!("[{}]:{}", server_ip, port);
    let key_bytes = read_key_file("key.txt")
        .expect("Client could not read key.txt");

    match input_file {
        Some(path) => { //if we have a filename then read it in and use as message to send to server
            let bytes = read_file_bytes(path)?;
            let text = String::from_utf8(bytes)
                .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "File is not valid UTF-8"))?;
            let text = text.lines().next().unwrap_or("").trim(); //was just .trim() before
            if text.len() > 250 {
                eprintln!("Message too long, max 250 characters");
                return Ok(());
            }
            send_and_receive(&socket, text, &server_addr, &key_bytes)?;
        }
        None => { //no filename given, then read from stdin in a loop until user types "quit" or empty line
            let stdin = io::stdin();
            let mut line = String::new(); 
            println!("Enter message: ");
            stdin.lock().read_line(&mut line)?; //use mutex to make other processes wait until we are done reading from stdin
            let line = line.trim(); 
            if line.contains('|'){
                eprintln!("message cannot contain the '|' character");
                return Ok(()); 
            }
            if line.len() > 250{
                eprintln!("Message too long, max 250 characters");
                return Ok(());
            }
            send_and_receive(&socket, line, &server_addr, &key_bytes)?;         
        }
    }
    Ok(())
}

// Encrypts and sends a single message to the server, then waits for and displays the response.
fn send_and_receive(
    socket: &UdpSocket,
    message: &str,
    server_addr: &str,
    key_bytes: &[u8],
) -> std::io::Result<()> {
    let encrypted = encrypt_message(message.as_bytes(), key_bytes);
    socket.send_to(&encrypted, server_addr)?;
    println!("Sent: {}", message);

    let mut buf = [0u8; MAX_PACKET_SIZE];
    match socket.recv_from(&mut buf) {
        Ok((amt, _)) => {
            let messages = deserialize_messages(&buf[..amt]);
            print_server_response(&messages);
        }
        Err(e) if e.kind() == io::ErrorKind::WouldBlock || e.kind() == io::ErrorKind::TimedOut => {
            eprintln!("No response from server after {}s, message may be lost", TIMEOUT_SECS);
        }
        Err(e) => return Err(e),
    }
    Ok(())
}

// Pads the message to even length and encrypts it in 16-bit blocks using the key file.
fn encrypt_message(data: &[u8], key_bytes: &[u8]) -> Vec<u8> {
    let mut padded = data.to_vec();
    if padded.len() % 2 != 0 {
        padded.push(0x00);
    }
    let mut encrypted = Vec::new();
    for (i, chunk) in padded.chunks(2).enumerate() {
        let block = ((chunk[0] as u16) << 8) | (chunk[1] as u16);
        let key = key_bytes[i % 8];
        let enc = encrypt_round(block, key);
        encrypted.push((enc >> 8) as u8);
        encrypted.push((enc & 0xFF) as u8);
    }
    encrypted
}

// Displays the last 5 messages received from the server with index, timestamp, IP, and content.
fn print_server_response(messages: &[BulletinMessage]) {
    println!("\n===== LAST {} MESSAGES FROM SERVER =====", messages.len());
    if messages.is_empty() {
        println!("  (no messages)");
    }
    for (i, msg) in messages.iter().enumerate() {
        println!("  [{}] {} | {} | {}", i + 1, format_timestamp(msg.timestamp), msg.sender_ip, msg.content);
    }
    println!("========================================\n");
}