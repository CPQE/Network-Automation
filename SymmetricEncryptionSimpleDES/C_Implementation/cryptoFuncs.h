#ifndef CRYPTO_FUNCS_H
#define CRYPTO_FUNCS_H

#include <stdint.h>

/**
 * encrypt_block - Encrypts a 16-bit block through 8 stages.
 * @block: 16-bit input (L is high byte, R is low byte).
 * @keys: Array of 8 stage keys.
 */
uint16_t encrypt_block(uint16_t block, uint8_t keys[8]);

/**
 * decrypt_block - Decrypts a 16-bit block through 8 stages.
 * @block: 16-bit ciphertext.
 * @keys: Array of 8 stage keys (applied in reverse).
 */
uint16_t decrypt_block(uint16_t block, uint8_t keys[8]);

#endif