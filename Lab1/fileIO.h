#ifndef FILE_IO_H
#define FILE_IO_H

#include <stdint.h>

/**
 * process_file - Reads input, applies crypto logic, writes output.
 * @mode: 1 for encrypt, 0 for decrypt.
 */
void process_file(const char* in_path, const char* out_path, uint8_t keys[8], int mode);

#endif