#include "cryptoFuncs.h"

// Logic: Swap (L->R, R->L) then XOR the new Right (original L) with K
static uint16_t encrypt_round(uint16_t input, uint8_t key) {
    uint8_t L = (input >> 8) & 0xFF;
    uint8_t R = input & 0xFF;

    uint8_t nextL = R;
    uint8_t nextR = L ^ key;

    return (uint16_t)((nextL << 8) | nextR);
}

// Inverse Logic: XOR the Right with K, then Swap (R->L, L->R)
static uint16_t decrypt_round(uint16_t input, uint8_t key) {
    uint8_t L_out = (input >> 8) & 0xFF;
    uint8_t R_out = input & 0xFF;

    uint8_t prevL = R_out ^ key;
    uint8_t prevR = L_out;

    return (uint16_t)((prevL << 8) | prevR);
}

uint16_t encrypt_block(uint16_t block, uint8_t keys[8]) {
    for (int i = 0; i < 8; i++) {
        block = encrypt_round(block, keys[i]);
    }
    return block;
}

uint16_t decrypt_block(uint16_t block, uint8_t keys[8]) {
    // Decryption uses keys in reverse order: K8, K7... K1
    for (int i = 7; i >= 0; i--) {
        block = decrypt_round(block, keys[i]);
    }
    return block;
}