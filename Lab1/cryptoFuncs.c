
#include "cryptoFuncs.h"
/**
 * des_stage - Executes a single iteration of the simplified DES algorithm.
 * @input: The 16-bit data block to process.
 * @key: The 8-bit key for this specific stage.
 * * Logic:
 * 1. Splits the 16-bit input into Left (L) and Right (R) bytes.
 * 2. Interchanges their places (L moves to right, R moves to left).
 * 3. Performs a bitwise XOR of the original L (now in the right position) and the key.
 * * Returns: The transformed 16-bit block.
 */
uint16_t des_stage(uint16_t input, uint8_t key) {
    uint8_t L = (input >> 8) & 0xFF;
    uint8_t R = input & 0xFF;

    // Interchange: Left becomes new Right, Right becomes new Left
    // Then XOR the original L with the key
    uint8_t nextL = R;
    uint8_t nextR = L ^ key;

    return (uint16_t)((nextL << 8) | nextR);
}

/**
 * encrypt_data - Encrypts a 16-bit block using 8 stages of S-DES.
 * @block: The 16-bit plaintext block.
 * @keys: An array of 8 keys (8 bits each).
 * * Logic:
 * Sequentially passes the data through the des_stage function 8 times
 * using keys in forward order (K[0] through K[7]).
 * * Returns: The encrypted 16-bit ciphertext block.
 */
uint16_t encrypt_data(uint16_t block, uint8_t keys[8]) {
    for (int i = 0; i < 8; i++) {
        block = des_stage(block, keys[i]);
    }
    return block;
}

/**
 * decrypt_data - Decrypts a 16-bit block using 8 stages of S-DES.
 * @block: The 16-bit ciphertext block.
 * @keys: An array of 8 keys (8 bits each).
 * * Logic:
 * Sequentially passes the data through the des_stage function 8 times
 * using keys in reverse order (K[7] down to K[0]). This reverse 
 * schedule undoes the Feistel transformations.
 * * Returns: The decrypted 16-bit plaintext block.
 */
uint16_t decrypt_data(uint16_t block, uint8_t keys[8]) {
    for (int i = 7; i >= 0; i--) {
        block = des_stage(block, keys[i]);
    }
    return block;
}