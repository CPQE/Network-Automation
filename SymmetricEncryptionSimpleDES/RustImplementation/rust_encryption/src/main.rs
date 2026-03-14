mod encryption;
mod decryption;
mod error_handling;
mod file_io;

use std::io;

fn main() -> io::Result<()> {
    // 1. Setup Data
    let key_data = file_io::read_raw_string("keys.txt")
        .expect("CRITICAL ERROR: keys.txt not found")
        .trim()
        .to_string();
    
    let raw_text = file_io::read_raw_string("plaintext.txt")
        .expect("CRITICAL ERROR: plaintext.txt not found");

    if key_data.len() < 8 {
        println!("Error: Key file must contain at least 8 characters.");
        return Ok(());
    }

    let key_bytes = key_data.as_bytes();

    // 2. Encryption Phase
    let blocks = error_handling::validate_and_format(&raw_text);
    let mut cipher_output = Vec::new();

    for b in blocks {
        let mut current_block = b;
        for i in 0..8 {
            current_block = encryption::encrypt_round(current_block, key_bytes[i]);
        }
        cipher_output.push(current_block);
    }

    file_io::write_encrypted_blocks("encrypted_cipher.txt", &cipher_output);
    println!("Encryption finished. Output in encrypted_cipher.txt");

    // 3. Decryption Phase
    let encrypted_blocks = file_io::read_encrypted_blocks("encrypted_cipher.txt");
    let mut decrypted_blocks = Vec::new();

    for cb in encrypted_blocks {
        let mut current_block = cb;
        // Decryption uses keys in reverse order
        for i in (0..8).rev() {
            current_block = decryption::decrypt_round(current_block, key_bytes[i]);
        }
        decrypted_blocks.push(current_block);
    }

    let final_output = error_handling::blocks_to_string(&decrypted_blocks);
    
    file_io::write_raw_string("decrypted_plaintext.txt", &final_output);
    println!("Decryption finished. Output in decrypted_plaintext.txt");

    Ok(())
}