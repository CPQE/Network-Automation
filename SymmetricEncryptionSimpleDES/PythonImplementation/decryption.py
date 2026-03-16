from encryption import swapBits, xor_key, combine_halves

def decrypt_round(data_block, key):
    # 1. Split into halves first
    l_data = (data_block >> 8) & 0xFF
    r_data = data_block & 0xFF
    # 2. Reverse the XOR (XORing with the same key undoes it)
    r_data = xor_key(r_data, key)
    # 3. Recombine to prepare for the bit swap
    data_block = combine_halves(l_data, r_data)
    # 4. Swap the 8-bit halves back (0 and 8 are the same positions used in encrypt)
    # This puts the original left side back on the left and right on the right
    original_block = swapBits(data_block, 0, 8, 8)
    return original_block

if __name__ == "__main__":
    # Test case using the result from your encryption
    encrypted_val = 0b1100110101011011 
    key = 0b11110000
    decrypted = decrypt_round(encrypted_val, key)
    print(f"Encrypted: {encrypted_val:016b}")
    print(f"Decrypted: {decrypted:016b}")