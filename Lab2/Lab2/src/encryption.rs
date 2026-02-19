pub fn swap_bits(x: u16, p1: u32, p2: u32, n: u32) -> u16 {
    //Swaps n bits starting at positions p1 and p2.
    let mask = (1u16 << n) - 1;
    let set1 = (x >> p1) & mask;
    let set2 = (x >> p2) & mask;
    let xor_val = set1 ^ set2;
    x ^ ((xor_val << p1) | (xor_val << p2))
}

pub fn xor_key(mut r_data: u8, key: u8) -> u8 {
    //xor's right half of the 16 bit block with the key
    r_data ^= key;
    r_data
}

pub fn combine_halves(l_data: u8, r_data: u8) -> u16 {
    //combines the 8 bit halves together back into a 16 bit one
    ((l_data as u16) << 8) | (r_data as u16)
}

pub fn encrypt_round(data_block: u16, key: u8) -> u16 {
    let swapped = swap_bits(data_block, 0, 8, 8); //swap lower and upper 8 bits
    let l_data = (swapped >> 8) as u8; //separate the left and right halves
    let r_data = (swapped & 0xFF) as u8;
    let r_data_xor = xor_key(r_data, key); //xor right half 
    combine_halves(l_data, r_data_xor) //combine them and return 16 bit result
}