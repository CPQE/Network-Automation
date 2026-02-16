use std::fs;

pub fn read_raw_string(filename: &str) -> Option<String> {
    fs::read_to_string(filename).ok()
}

pub fn write_raw_string(filename: &str, data: &str) {
    let _ = fs::write(filename, data);
}

pub fn write_encrypted_blocks(filename: &str, blocks: &[u16]) {
    let content = blocks
        .iter()
        .map(|b| b.to_string())
        .collect::<Vec<_>>()
        .join(",");
    let _ = fs::write(filename, content);
}

pub fn read_encrypted_blocks(filename: &str) -> Vec<u16> {
    fs::read_to_string(filename)
        .unwrap_or_default()
        .split(',')
        .filter_map(|s| s.parse::<u16>().ok())
        .collect()
}