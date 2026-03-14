pub fn validate_and_format(raw_input: &str) -> Vec<u16> {
    if raw_input.is_empty() {
        println!("ALERT: No data found to process.");
        return Vec::new();
    }

    let mut data_bytes = raw_input.as_bytes().to_vec();
    if data_bytes.len() % 2 != 0 {
        data_bytes.push(0);
    }

    data_bytes
        .chunks(2)
        .map(|chunk| ((chunk[0] as u16) << 8) | (chunk[1] as u16))
        .collect()
}

pub fn blocks_to_string(blocks: &[u16]) -> String {
    let mut bytes = Vec::new();
    for &b in blocks {
        bytes.push((b >> 8) as u8);
        bytes.push((b & 0xFF) as u8);
    }
    String::from_utf8_lossy(&bytes)
        .trim_end_matches('\0')
        .to_string()
}