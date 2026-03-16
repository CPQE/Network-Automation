use std::env;
use std::time::{SystemTime, UNIX_EPOCH};
use circular_buffer::CircularBuffer;
use chrono::{DateTime, Utc};

pub struct BulletinMessage {
    pub content: String,      // decrypted message text
    pub sender_ip: String,    // e.g. "192.168.1.5"
    pub timestamp: u64,       // unix timestamp seconds
}

pub fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

pub fn format_timestamp(ts: u64) -> String {
    let dt = DateTime::<Utc>::from_timestamp(ts as i64, 0)
        .unwrap_or_default();
    dt.format("%b %e %H:%M:%S").to_string()
}

pub enum Mode {
    Sender {
        data_file: String,
        src_ip: String,
        dst_ip: String,
        src_port: u16,
        dst_port: u16,
        output_file: String,
    },
    Receiver {
        src_ip: String,
        dst_ip: String,
        datagram_file: String,
    },
     Client {
        server_ip: String,
        port: u16,
        input_file: Option<String>, // None means read from stdin
    },
    Server {
        port: u16,
    },
}
    
pub fn parse_args() -> Result<Mode, String> {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() < 2 {
        return Err("Usage: \n  sender <data_file> <src_ip> <dst_ip> <src_port> <dst_port> <output_file>\n  receiver <src_ip> <dst_ip> <datagram_file>\n  server <port>\n  client <server_ip> <port> [input_file]".into());
    }

    match args[1].as_str() {
        "sender" => {
            if args.len() != 8 {
                return Err("Sender usage: sender <data_file> <src_ip> <dst_ip> <src_port> <dst_port> <output_file>".into());
            }
            Ok(Mode::Sender {
                data_file:   args[2].clone(),
                src_ip:      args[3].clone(),
                dst_ip:      args[4].clone(),
                src_port:    args[5].parse::<u16>().map_err(|_| "Invalid src_port")?,
                dst_port:    args[6].parse::<u16>().map_err(|_| "Invalid dst_port")?,
                output_file: args[7].clone(),
            })
        }
        "receiver" => {
            if args.len() != 5 {
                return Err("Receiver usage: receiver <src_ip> <dst_ip> <datagram_file>".into());
            }
            Ok(Mode::Receiver {
                src_ip:        args[2].clone(),
                dst_ip:        args[3].clone(),
                datagram_file: args[4].clone(),
            })
        }
        "server" => {
            if args.len() != 3 {
                return Err("Server usage: server <port>".into());
            }
            Ok(Mode::Server {
                port: args[2].parse::<u16>().map_err(|_| "Invalid port")?,
            })
        }
        "client" => {
            if args.len() < 4 || args.len() > 5 {
                return Err("Client usage: client <server_ip> <port> [input_file]".into());
            }
            Ok(Mode::Client {
                server_ip:  args[2].clone(),
                port:       args[3].parse::<u16>().map_err(|_| "Invalid port")?,
                input_file: args.get(4).cloned(), // Some(filename) or None for stdin
            })
        }
        _ => Err(format!("Unknown mode '{}'. Expected: sender, receiver, server, client", args[1])).into(),
    }
}

pub fn parse_ip(ip: &str) -> Result<[u8; 4], String> {
    let parts: Vec<&str> = ip.split('.').collect(); //make input string into string array demilited by .
    if parts.len() != 4 { //throw error if not right size
        return Err(format!("Invalid IP '{}': must contain 4 octets", ip));
    }
    let mut octets = [0u8; 4]; //make fixed size array of 4 bytes initialized to 00000000
    for (i, part) in parts.iter().enumerate() {
        let value = part
            .parse::<u8>() //attempts to convert each octet into unsigned 8 bit integer
            .map_err(|_| format!("Invalid octet '{}' in IP '{}'", part, ip))?;
        octets[i] = value; //insert octet into octets array
    }
    Ok(octets) //final result like [192, 168, 0, 1]
}

 pub fn ip_to_u32_be(octets: [u8; 4]) -> u32 {
    //convert 4 octets into single big-endian 32 bit unsigned integer
    ((octets[0] as u32) << 24) |
     ((octets[1] as u32) << 16) |
      ((octets[2] as u32) << 8) | 
      (octets[3] as u32) 
  }

pub fn u16_to_be_bytes(value: u16) -> [u8; 2] {
    [   
        // high byte is the leftmost 8 bits, low byte is the rightmost 8 bits
        (value >> 8) as u8,   // high byte
        (value & 0xFF) as u8, // low byte
    ]
}

pub fn u32_to_be_bytes(value: u32) -> [u8; 4] {
    [
        // big-endian means the most significant byte is first
        ((value >> 24) & 0xFF) as u8,
        ((value >> 16) & 0xFF) as u8,
        ((value >> 8)  & 0xFF) as u8,
        (value & 0xFF) as u8,
    ]
}

pub fn be_bytes_to_u16(bytes: [u8; 2]) -> u16 {
    // big-endian means the first byte is the high byte and the second byte is the low byte
    ((bytes[0] as u16) << 8) | (bytes[1] as u16)
}

pub fn pad_data_if_needed(data: &mut Vec<u8>) {
    // If the length of the data is odd, we need to add a padding byte (0x00) at the end
    if data.len() % 2 != 0 {
        data.push(0x00);
    }
}

// Serialize the last 5 messages into a single byte buffer to send over UDP
// Format per message: "IP|TIMESTAMP|CONTENT\n"
pub fn serialize_messages(messages: &CircularBuffer<5, BulletinMessage>) -> Vec<u8> {
    let mut out = String::new();
    for msg in messages {
        out.push_str(&format!("{}|{}|{}\n", msg.sender_ip, msg.timestamp, msg.content));
    }
    out.into_bytes()
}

// Deserialize the buffer back into a vec of BulletinMessages on the client side
pub fn deserialize_messages(data: &[u8]) -> Vec<BulletinMessage> {
    let text = String::from_utf8_lossy(data);
    let mut messages = Vec::new();
    for line in text.lines() {
        let parts: Vec<&str> = line.splitn(3, '|').collect();
        if parts.len() == 3 {
            messages.push(BulletinMessage {
                sender_ip: parts[0].to_string(),
                timestamp: parts[1].parse::<u64>().unwrap_or(0),
                content: parts[2].to_string(),
            });
        }
    }
    messages
}