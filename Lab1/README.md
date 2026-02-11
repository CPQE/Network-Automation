# Simplified DES (S-DES) Implementation

This program implements a symmetric key block cipher based on a simplified version of the Data Encryption Standard (DES). It processes data in 16-bit blocks using an 8-stage Feistel-inspired network.

## Modular Design

The implementation follows a modular architecture to ensure clarity and mathematical symmetry between encryption and decryption operations.

### 1. Data Splitting and Transformation
The 16-bit input block is treated as two distinct 8-bit fields:
- **L (Left):** The high-order byte (bits 15-8).
- **R (Right):** The low-order byte (bits 7-0).

During each stage, the bytes are interchanged (swapped) and a bitwise XOR operation is performed between the original left byte and the stage-specific key to produce the new right byte.

### 2. The 8-Stage Pipeline
The security of the cipher is derived from repeating the transformation 8 times. 
- **Encryption:** Uses the keys in the order provided ($K_1, K_2, \dots, K_8$).
- **Decryption:** Uses the same keys in reverse order ($K_8, K_7, \dots, K_1$).

Applying the keys in reverse order effectively undoes the XOR and swap operations, restoring the original plaintext.

---

## Function Documentation

### `uint16_t des_stage(uint16_t input, uint8_t key)`
This function performs a single iteration (round) of the cipher.
- **Parameters:**
    - `input`: The 16-bit data block to be transformed.
    - `key`: The 8-bit sub-key for the current stage.
- **Logic:**
    1. Extracts `L` and `R` using bit-masking and shifting.
    2. Moves the original `R` to the left position.
    3. Performs `L XOR key` and moves the result to the right position.
- **Returns:** The resulting 16-bit block.

### `void process_file(const char* in_path, const char* out_path, uint8_t keys[8], int encrypt)`
The primary controller for file stream processing and round coordination.
- **Parameters:**
    - `in_path`: Path to the source file.
    - `out_path`: Path to the destination file.
    - `keys`: An array of eight 8-bit keys.
    - `encrypt`: Integer flag (1 for encryption, 0 for decryption).
- **Logic:**
    - Opens files in binary mode (`"rb"` and `"wb"`) to prevent newline translations.
    - Reads 16 bits at a time into a buffer.
    - Executes a loop of 8 calls to `des_stage`.
    - Selects the key index dynamically based on the `encrypt` flag to ensure the correct key schedule.

---

## File Specifications

### Key File Format
The key file must contain 8 characters. Each character represents one 8-bit key. For example, a file containing `12345678` provides:
- Key 1: '1' (0x31)
- Key 2: '2' (0x32)
- ...and so on.

### Input/Output
- **Encryption:** Reads plaintext (Data) and outputs a binary ciphertext file.
- **Decryption:** Reads the binary ciphertext and outputs the restored plaintext.

## Compilation and Execution

### Testing: 
Encrypt: python sdes.py encrypt plaintext.txt ciphertext.bin keys.txt
Decrypt: python sdes.py decrypt ciphertext.bin restored.txt keys.txt