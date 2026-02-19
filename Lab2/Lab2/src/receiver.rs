
use std::io;

use crate::decryption::decrypt_round;
use crate::file_io::{read_file_bytes, read_key_file};
use crate::headers::{build_pseudo_header, build_udp_header_without_checksum};
use crate::utilities::{be_bytes_to_u16, u16_to_be_bytes, pad_data_if_needed};
use crate::checksum::calculate_packet_checksum;

pub struct ParsedUdpPacket {
    pub src_port: u16,
    pub dst_port: u16,
    pub length: u16,
    pub checksum: u16,
    pub payload: Vec<u8>, //decrypted data from payload
}
//receiver has to read datagram file, parse UDP header, verify checksum, extract payload, and write payload to output file.

pub fn parse_udp_datagram(
    path: &str,
    src_ip_u32: u32,
    dst_ip_u32: u32,
) -> io::Result<ParsedUdpPacket> {

    // 1. Read datagram
    let bytes = read_file_bytes(path)?;
    if bytes.len() < 8 {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "Datagram too short"));
    }
    // 2. Parse header
    let src_port = be_bytes_to_u16([bytes[0], bytes[1]]);
    let dst_port = be_bytes_to_u16([bytes[2], bytes[3]]);
    let length   = be_bytes_to_u16([bytes[4], bytes[5]]);
    let checksum = be_bytes_to_u16([bytes[6], bytes[7]]);
    // 3. Extract payload (still encrypted)
    let encrypted_payload = bytes[8..].to_vec();
    // 4. Recompute checksum
    let mut udp_header_zero = build_udp_header_without_checksum(src_port, dst_port, length);
    udp_header_zero[6] = 0;
    udp_header_zero[7] = 0;
    let pseudo = build_pseudo_header(src_ip_u32, dst_ip_u32, length);
    let mut checksum_data = Vec::new();
    checksum_data.extend_from_slice(&pseudo);
    checksum_data.extend_from_slice(&udp_header_zero);
    checksum_data.extend_from_slice(&encrypted_payload);
    let mut padded = checksum_data.clone();
    pad_data_if_needed(&mut padded);
    let computed = calculate_packet_checksum(&padded);
    if computed != checksum {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Checksum mismatch: expected {:#06X}, got {:#06X}", checksum, computed),
        ));
    }
    let key_bytes= read_key_file("key.txt")?;
    let mut decrypted = Vec::new();
    for (i, chunk) in encrypted_payload.chunks(2).enumerate(){
        let block = ((chunk[0] as u16) << 8) | (chunk[1] as u16);
        let key =   key_bytes[i % 8]; //cycle through 8 byte key
        let dec = decrypt_round(block, key); 
        decrypted.push((dec >> 8) as u8);
        decrypted.push((dec & 0xFF) as u8); 
    }
    // Remove trailing null padding if present
    let mut final_payload = decrypted;
    while final_payload.last() == Some(&0) {
        final_payload.pop();
    }

    Ok(ParsedUdpPacket {
        src_port,
        dst_port,
        length,
        checksum,
        payload: final_payload,
    })
}

pub fn write_payload_to_file(path: &str, data: &[u8]) -> io::Result<()> {
    std::fs::write(path, data)
}
