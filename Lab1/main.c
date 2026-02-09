#include "fileIO.h"
#include "cryptoFuncs.h"

int main(int argc, char *argv[]) {
    if (argc < 5) {
        printf("Usage: %s <encrypt|decrypt> <input_file> <output_file> <key_file>\n", argv[0]);
        return 1;
    }

    // Read keys from file (1 key = 1 character)
    uint8_t keys[8];
    FILE *key_file = fopen(argv[4], "r");
    if (!key_file) {
        perror("Key file error");
        return 1;
    }
    
    for (int i = 0; i < 8; i++) {
        int ch = fgetc(key_file);
        if (ch == EOF) {
            fprintf(stderr, "Error: Key file must contain at least 8 characters.\n");
            return 1;
        }
        keys[i] = (uint8_t)ch;
    }
    fclose(key_file);

    process_files(argv[1], argv[2], argv[3], keys);

    return 0;
}