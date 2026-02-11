import sys

def encrypt_round(data_block, key):
    """L moves to R, R moves to L, then the new R (old L) is XORed with K."""
    L = (data_block >> 8) & 0xFF
    R = data_block & 0xFF
    
    next_L = R
    next_R = L ^ key
    return (next_L << 8) | next_R

def decrypt_round(data_block, key):
    """The mathematical inverse of the encryption round."""
    L_out = (data_block >> 8) & 0xFF
    R_out = data_block & 0xFF
    # Algebra: 
    # R_out = L_in ^ key  => L_in = R_out ^ key
    # L_out = R_in       => R_in = L_out
    prev_L = R_out ^ key
    prev_R = L_out
    return (prev_L << 8) | prev_R

def process_file(mode, input_fn, output_fn, keys):
    try:
        with open(input_fn, 'rb') as f_in, open(output_fn, 'wb') as f_out:
            while True:
                chunk = f_in.read(2)
                if not chunk:
                    break
                
                # Padding: If file size is odd, add a null byte
                if len(chunk) == 1:
                    chunk += b'\x00'
                
                # Convert bytes to 16-bit integer (Big Endian)
                block = int.from_bytes(chunk, byteorder='big')
                
                if mode == 'encrypt':
                    for k in keys:
                        block = encrypt_round(block, k)
                else:
                    # Decryption uses keys in reverse (K8 down to K1)
                    for k in reversed(keys):
                        block = decrypt_round(block, k)
                
                # Write back as 2 bytes
                f_out.write(block.to_bytes(2, byteorder='big'))
                
        print(f"Success: {mode}ed '{input_fn}' into '{output_fn}'")
    except Exception as e:
        print(f"Error: {e}")

def main():
    if len(sys.argv) < 5:
        print("Usage: python sdes.py <encrypt|decrypt> <input> <output> <keyfile>")
        return

    mode = sys.argv[1].lower()
    input_file = sys.argv[2]
    output_file = sys.argv[3]
    key_file = sys.argv[4]

    # Read the 8-character key file
    try:
        with open(key_file, 'rb') as kf:
            keys = list(kf.read(8))
            if len(keys) < 8:
                print("Error: Key file must contain at least 8 characters.")
                return
    except FileNotFoundError:
        print("Error: Key file not found.")
        return

    process_file(mode, input_file, output_file, keys)

if __name__ == "__main__":
    main()