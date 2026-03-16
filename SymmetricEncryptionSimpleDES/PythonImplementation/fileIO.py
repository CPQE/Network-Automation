import os

def read_raw_string(filename):
    try:
        with open(filename, 'r') as f:
            content = f.read().strip()
            if not content:
                raise ValueError(f"{filename} is empty")
            return content
    except FileNotFoundError:
        print(f"CRITICAL ERROR: The file '{filename}' was not found.")
        return None
    except Exception as e:
        print(f"SYSTEM ERROR: Could not read {filename}: {e}")
        return None
def write_raw_string(filename, data):
    try:
        with open(filename, 'w') as f:
            f.write(data)
    except PermissionError:
        print(f"CRITICAL ERROR: Permission denied writing to {filename}.")
        
def write_raw_string(filename, data):
    with open(filename, 'w') as f:
        f.write(data)

def write_encrypted_blocks(filename, blocks):
    with open(filename, 'w') as f:
        # Saving as comma-separated integers to avoid encoding issues
        f.write(",".join(map(str, blocks)))

def read_encrypted_blocks(filename):
    if not os.path.exists(filename):
        return []
    with open(filename, 'r') as f:
        content = f.read().strip()
        return [int(b) for b in content.split(",")] if content else []