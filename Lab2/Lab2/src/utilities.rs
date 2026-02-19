use std::env;

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
    } 
}

pub fn parse_args() -> Result<Mode, String> {
    let args: Vec<String> = std::env::args().collect();
    match args.len() {
        7 => {         // Sender mode
            let data_file = args[1].clone();
            let src_ip = args[2].clone();
            let dst_ip = args[3].clone();
            let src_port = args[4].parse::<u16>().map_err(|_| "Invalid src_port")?;
            let dst_port = args[5].parse::<u16>().map_err(|_| "Invalid dst_port")?;
            let output_file = args[6].clone();

            Ok(Mode::Sender {
                data_file,
                src_ip,
                dst_ip,
                src_port,
                dst_port,
                output_file,
            })
        }

        // Receiver mode
        4 => {
            let src_ip = args[1].clone();
            let dst_ip = args[2].clone();
            let datagram_file = args[3].clone();

            Ok(Mode::Receiver {
                src_ip,
                dst_ip,
                datagram_file,
            })
        }

        _ => Err("Invalid number of arguments".into()),
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
