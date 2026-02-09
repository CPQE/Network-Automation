#ifndef CRYPTO_FUNCS_H
#define CRYPTO_FUNCS_H
#include <stdio.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>

/**
 * des_stage - Executes a single iteration of the simplified DES algorithm.
 * @input: The 16-bit data block to process.
 * @key: The 8-bit key for this specific stage.
 *
 * Performs the swap and XOR logic: L becomes R, and R becomes L ^ key.
 * Returns the transformed 16-bit block.
 */
uint16_t des_stage(uint16_t input, uint8_t key);

/**
 * encrypt_data - Encrypts a 16-bit block using 8 stages of S-DES.
 * @block: The 16-bit plaintext block.
 * @keys: An array of 8 keys (8 bits each).
 *
 * Processes the block using the keys in forward order (0 to 7).
 */
uint16_t encrypt_data(uint16_t block, uint8_t keys[8]);

/**
 * decrypt_data - Decrypts a 16-bit block using 8 stages of S-DES.
 * @block: The 16-bit ciphertext block.
 * @keys: An array of 8 keys (8 bits each).
 *
 * Processes the block using the keys in reverse order (7 down to 0).
 */
uint16_t decrypt_data(uint16_t block, uint8_t keys[8]);

#endif