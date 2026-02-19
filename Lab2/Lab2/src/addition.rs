
pub fn calculate_packet_checksum(data: &[u8]) -> u16 {
    let mut words = Vec::new();

    // Convert bytes to u16 words 
    for chunk in data.chunks(2) {
        let word = if chunk.len() == 2 {
            ((chunk[0] as u16) << 8) | (chunk[1] as u16)
        } else {
            (chunk[0] as u16) << 8
        };
        words.push(word);
    }

    // We start with 0 and add every word one by one 
    let final_sum = words.iter().fold(0, |acc, &word| {
        add_ones_complement(acc, word)
    });

    // bit flip at the very end (get 1's complement)
    !final_sum
}

pub fn add_ones_complement(a: u16, b: u16) -> u16 {
    let sum = a as u32 + b as u32;
    if sum > 0xFFFF {
        (sum & 0xFFFF) as u16 + 1
    } else {
        sum as u16
    }
}