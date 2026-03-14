use crate::encryption::{swap_bits, xor_key, combine_halves};

pub fn decrypt_round(data_block: u16, key: u8) -> u16 {
    let l_data = (data_block >> 8) as u8; //split into left and right halves
    let r_data = (data_block & 0xFF) as u8;
    
    let r_data_decrypted = xor_key(r_data, key); //xor (to undo encryption xor)
    let combined = combine_halves(l_data, r_data_decrypted); //recombine 
    
    swap_bits(combined, 0, 8, 8) //swap halves back to get original 16 bit data block
}