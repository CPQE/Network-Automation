#include <stdio.h>
#include "fileIO.h"
#include "cryptoFuncs.h"

void process_file(const char* in_path, const char* out_path, uint8_t keys[8], int mode) {
    FILE *in = fopen(in_path, "rb");
    FILE *out = fopen(out_path, "wb");
    if (!in || !out) { perror("File IO error"); return; }

    uint8_t buffer[2];
    // Read 2 bytes at a time
    while (fread(buffer, 1, 2, in) == 2) {
        // Construct 16-bit block: first byte is Left, second is Right
        uint16_t block = (uint16_t)((buffer[0] << 8) | buffer[1]);

        if (mode == 1) {
            block = encrypt_block(block, keys);
        } else {
            block = decrypt_block(block, keys);
        }

        // Deconstruct back to bytes for output
        uint8_t out_bytes[2];
        out_bytes[0] = (uint8_t)(block >> 8);
        out_bytes[1] = (uint8_t)(block & 0xFF);
        fwrite(out_bytes, 1, 2, out);
    }
    
    fclose(in);
    fclose(out);
}