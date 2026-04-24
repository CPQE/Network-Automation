import os
import random
import string
import socket
OWN_DIR = "./own"
NUM_FILES = 5
FILE_SIZE_RANGE = (100, 500)

COMMON_WORDS = [
    "network", "peer", "data", "packet", "link", "file", "query", "message",
    "node", "client", "server", "udp", "tcp", "join", "leave", "hash",
    "route", "send", "receive", "connect", "disconnect", "socket", "transfer"
]

os.makedirs(OWN_DIR, exist_ok=True)

# Remove existing files in ./own directory
for filename in os.listdir(OWN_DIR):
    file_path = os.path.join(OWN_DIR, filename)
    if os.path.isfile(file_path):
        os.remove(file_path)
    print(f"Removed file: {file_path}")

def get_hostname_suffix():
    hostname = socket.gethostname()
    return ''.join(filter(str.isalnum, hostname))

def generate_random_filename(suffix):
    words = random.sample(COMMON_WORDS, 2)
    return f"{words[0]}_{words[1]}_{suffix}.txt"

def generate_random_content(size):
    words = []
    total_len = 0
    while total_len < size:
        word = random.choice(COMMON_WORDS)
        words.append(word)
        total_len += len(word) + 1
    return ' '.join(words)

def main():
    suffix = get_hostname_suffix()
    for _ in range(NUM_FILES):
        filename = generate_random_filename(suffix)
        filepath = os.path.join(OWN_DIR, filename)
        size = random.randint(*FILE_SIZE_RANGE)
        content = generate_random_content(size)
        with open(filepath, 'w') as f:
            f.write(content)
        print(f"Generated file: {filepath} ({size} bytes)")

if __name__ == "__main__":
    main()
