#DES Encryption Module
def swapBits(x, p1, p2, n):
    '''within the 16-bit block swap n bits to the left of the starting point p1 (with 0 being rightmost bit) 
    with the bits in the second set (starting at p2, beginning at rightmost bit of 0 index)'''
    # Move all bits of first set to rightmost side 
    set1 = (x >> p1) & ((1 << n) - 1)
    # Move all bits of second set to rightmost side 
    set2 = (x >> p2) & ((1 << n) - 1)
    # Xor the two sets 
    xorVal = (set1 ^ set2)
    # Put the Xor bits back to their original positions 
    xorVal = (xorVal << p1) | (xorVal << p2)
    # Xor the 'Xor' with the original number so that the 
    # two sets are swapped 
    result = x ^ xorVal
    # print(f"Final binary result after swap: {result:016b}")  # Output: '0b11001'
    return result

def xor_key(r_data, key):
    '''XOR the right half with the key.'''
    r_data ^= key
    return r_data 

def combine_halves(l_data, r_data):
    '''combine left half with right half for final part of encryption round'''
    result = (l_data << 8) | r_data
    return result

def encrypt_round(data_block, key):
    '''4 stage process, swap halves, xor right half with key, combine halves for final output'''
    data_block = swapBits(data_block, 0, 8, 8)
    l_data = (data_block >> 8) & 0xFF
    r_data = data_block & 0xFF
    r_data = xor_key(r_data, key)
    ret_data = combine_halves(l_data, r_data)
    return ret_data

if __name__ == "__main__":
    test_block = 0b1010101111001101 
    test_key = 0b11110000
    encrypted = encrypt_round(test_block, test_key)
    # 3. Print the results in binary to verify the bits
    print(f"Input:     {test_block:016b}")
    print(f"Key:       {test_key:08b}")
    print(f"Result:    {encrypted:016b}")
#source for swap: https://www.geeksforgeeks.org/dsa/swap-bits-in-a-given-number/