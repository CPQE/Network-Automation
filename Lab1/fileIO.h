#ifndef FILE_IO_H
#define FILE_IO_H

#include <stdint.h>
#include <stdio.h>

/**
 * process_files - Handles the I/O loop for encryption and decryption.
 * @mode: String indicating "encrypt" or "decrypt".
 * @input_fn: Path to the source file.
 * @output_fn: Path to the destination file.
 * @keys: The 8-bit key array retrieved from the key file.
 *
 * Reads 16-bit chunks, processes them via cryptoFuncs, and writes to disk.
 */
void process_files(const char* mode, const char* input_fn, const char* output_fn, uint8_t keys[8]);

#endif