
def validate_and_format(raw_input):
    # Requirement: Handle missing or invalid inputs
    if raw_input is None or raw_input == "":
        # Instead of a silent [0], return an empty list 
        # so the 'for b in blocks' loop doesn't run at all.
        print("ALERT: No data found to process.")
        return []

    if isinstance(raw_input, str):
        data_bytes = raw_input.encode('utf-8')
        # Handle odd/even bytes requirement
        if len(data_bytes) % 2 != 0:
            data_bytes += b'\x00'
            
        blocks = []
        for i in range(0, len(data_bytes), 2):
            blocks.append((data_bytes[i] << 8) | data_bytes[i+1])
        return blocks
    
    return [raw_input] if isinstance(raw_input, int) else []




def validate_key(key):
    '''Ensures the key fits in 8 bits (0-255)'''
    try:
        k = int(key)
        return k & 0xFF # Mask it to force it into 8 bits
    except (ValueError, TypeError):
        print("Invalid key format. Using default key 0xAA")
        return 0xAA
    
def validate_key_length(key_string, required_rounds=8):
    """Ensures the key is long enough for the number of rounds."""
    if not key_string:
        return False, "Key is missing or empty."
    if len(key_string) < required_rounds:
        return False, f"Key is too short. Need {required_rounds} chars, got {len(key_string)}."
    return True, "Success"


def blocks_to_string(blocks):
    """
    Converts a list of 16-bit integers back into a UTF-8 string.
    It splits each integer into two bytes and strips trailing null padding.
    """
    result_bytes = bytearray()
    for b in blocks:
        # Extract the high byte (left 8 bits)
        result_bytes.append((b >> 8) & 0xFF)
        # Extract the low byte (right 8 bits)
        result_bytes.append(b & 0xFF)

    # Decode to string and strip the \x00 padding we added in validate_and_format
    return result_bytes.decode('utf-8').rstrip('\x00')