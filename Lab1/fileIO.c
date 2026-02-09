
#include "fileIO.h"

void process_files(const char* mode, const char* input_fn, const char* output_fn, uint8_t keys[8]) {
    FILE *in = fopen(input_fn, "rb");
    FILE *out = fopen(output_fn, "wb");
    
    if (!in || !out) {
        perror("File opening failed");
        return;
    }

    uint16_t buffer;
    int is_encrypt = (strcmp(mode, "encrypt") == 0);

    // Read 16 bits (2 bytes) at a time
    while (fread(&buffer, sizeof(uint16_t), 1, in) == 1) {
        if (is_encrypt) {
            buffer = encrypt_data(buffer, keys);
        } else {
            buffer = decrypt_data(buffer, keys);
        }
        fwrite(&buffer, sizeof(uint16_t), 1, out);
    }

    fclose(in);
    fclose(out);
}