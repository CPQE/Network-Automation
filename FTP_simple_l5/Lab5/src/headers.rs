//contains functions used to build udp headers and pseudo-headers
use crate::utilities::{u16_to_be_bytes, u32_to_be_bytes};

pub fn build_udp_header_without_checksum(src_port: u16, dst_port: u16, length: u16) -> [u8; 8] {
    let sp = u16_to_be_bytes(src_port);
    let dp = u16_to_be_bytes(dst_port);
    let len = u16_to_be_bytes(length);
    [
        sp[0], sp[1],
        dp[0], dp[1],
        len[0], len[1],
        0x00, 0x00, // checksum placeholder
    ]
}

pub fn build_pseudo_header(src_ip: u32, dst_ip: u32, udp_length: u16) -> Vec<u8> {
    let mut v = Vec::with_capacity(12);
    let s = u32_to_be_bytes(src_ip);
    let d = u32_to_be_bytes(dst_ip);
    let l = u16_to_be_bytes(udp_length);
    v.extend_from_slice(&s);
    v.extend_from_slice(&d);
    v.push(0x00);      // zero
    v.push(17);        // protocol = UDP
    v.extend_from_slice(&l);
    v
}
