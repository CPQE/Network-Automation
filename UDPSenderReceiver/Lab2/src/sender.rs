use std::io;
use crate::encryption::encrypt_round;
use crate::file_io::read_file_bytes;
use crate::headers::{build_udp_header_without_checksum, build_pseudo_header};
use crate::utilities::{pad_data_if_needed, u16_to_be_bytes};
use crate::checksum::calculate_packet_checksum;
use crate::file_io::read_key_file; 

pub fn build_udp_datagram(
    data_file: &str,
    src_ip_u32: u32,
    dst_ip_u32: u32,
    src_port: u16,
    dst_port: u16,
) -> io::Result<Vec<u8>> {
    // 1. Read payload
    let mut payload = read_file_bytes(data_file)?;
    // 2. Pad if needed
    pad_data_if_needed(&mut payload);
    
    // 2. Pad if needed
    pad_data_if_needed(&mut payload);

    // 2.5 Encrypt payload (16-bit blocks)
    let key_bytes = read_key_file("key.txt")?;
    let mut encrypted = Vec::new();
    for (i, chunk) in payload.chunks(2).enumerate() {
        let block = ((chunk[0] as u16) << 8) | (chunk[1] as u16);
        // cycle through the 8-byte key
        let key = key_bytes[i % 8];
        let enc = encrypt_round(block, key);
        encrypted.push((enc >> 8) as u8);
        encrypted.push((enc & 0xFF) as u8);
    }
    payload = encrypted;
    // 3. Compute UDP length
    let udp_length = (8 + payload.len()) as u16;
    // 4. Build UDP header with checksum = 0
    let mut udp_header = build_udp_header_without_checksum(
        src_port,
        dst_port,
        udp_length,
    );
    // 5. Build pseudo-header
    let pseudo = build_pseudo_header(
        src_ip_u32,
        dst_ip_u32,
        udp_length,
    );
    // 6. Build checksum buffer
    let mut checksum_data = Vec::new();
    checksum_data.extend_from_slice(&pseudo);
    checksum_data.extend_from_slice(&udp_header);
    checksum_data.extend_from_slice(&payload);
    // 7. Compute checksum
    let checksum = calculate_packet_checksum(&checksum_data);
    // 8. Insert checksum into UDP header
    let csum_bytes = u16_to_be_bytes(checksum);
    udp_header[6] = csum_bytes[0];
    udp_header[7] = csum_bytes[1];
    // 9. Build final datagram
    let mut datagram = Vec::new();
    datagram.extend_from_slice(&udp_header);
    datagram.extend_from_slice(&payload);
    Ok(datagram)
}

pub fn write_datagram_to_file(path: &str, data: &[u8]) -> io::Result<()> { 
    std::fs::write(path, data) 
}

