#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "fileIO.h"

int main(int argc, char *argv[]) {
    if (argc < 5) {
        printf("Usage: sdes <encrypt|decrypt> <input> <output> <keyfile>\n");
        return 1;
    }

    uint8_t keys[8];
    FILE *kf = fopen(argv[4], "rb");
    if (!kf) { perror("Keyfile error"); return 1; }
    
    // Read exactly 8 characters as keys
    if (fread(keys, 1, 8, kf) < 8) {
        fprintf(stderr, "Error: Keyfile must be at least 8 bytes.\n");
        return 1;
    }
    fclose(kf);

    int mode = (strcmp(argv[1], "encrypt") == 0) ? 1 : 0;
    process_file(argv[2], argv[3], keys, mode);

    return 0;
}