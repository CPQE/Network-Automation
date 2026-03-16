
from encryption import encrypt_round
from decryption import decrypt_round
from error_handling import validate_and_format, validate_key
from error_handling import validate_and_format, blocks_to_string
import fileIO
# Assuming encrypt_round and decrypt_round are imported or defined above
def encrypt_loop(plaintext, key_string):
    # Perform 8 rounds, using one char of the key per round
    for i in range(8):
        round_key = ord(key_string[i])
        plaintext = encrypt_round(plaintext, round_key)
    return plaintext

def decrypt_loop(ciphertext, key_string):
    # Decrypting requires using the keys in REVERSE order (from index 7 down to 0)
    for i in range(7, -1, -1):
        round_key = ord(key_string[i])
        ciphertext = decrypt_round(ciphertext, round_key)
    return ciphertext

def main():
    # 1. Setup Data from Files
    # keys.txt contains ";lkjhgfd"
    # plaintext.txt contains "a1b=p"
    key_data = fileIO.read_raw_string("keys.txt")
    raw_text = fileIO.read_raw_string("plaintext.txt")
    if len(key_data) < 8:
        print("Error: Key file must contain at least 8 characters.")
        return
    # 2. Encryption Phase
    # validate_and_format converts "a1b=p" -> three 16-bit blocks (with 1 null byte padding)
    blocks = validate_and_format(raw_text)
    cipher_output = []
    for b in blocks:
        cipher_output.append(encrypt_loop(b, key_data))
    fileIO.write_encrypted_blocks("encrypted_cipher.txt", cipher_output)
    print("Encryption finished. Output in encrypted_cipher.txt")
    # 3. Decryption Phase
    encrypted_blocks = fileIO.read_encrypted_blocks("encrypted_cipher.txt")
    decrypted_blocks = []
    for cb in encrypted_blocks:
        decrypted_blocks.append(decrypt_loop(cb, key_data))
    # blocks_to_string strips the padding added during encryption
    final_output = blocks_to_string(decrypted_blocks)
    

    print(f"DEBUG: Decrypted Blocks: {decrypted_blocks}")
    final_output = blocks_to_string(decrypted_blocks)
    print(f"DEBUG: Final String: '{final_output}'")

    fileIO.write_raw_string("decrypted_plaintext.py", final_output)
    print("Decryption finished. Output in decrypted_plaintext.py")

if __name__ == "__main__":
    main()