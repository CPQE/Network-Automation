UDP Encrypted Bulletin Board

Rust implementation of an encrypted UDP client-server bulletin board system, built on top of a previous assignment's encryption and checksum infrastructure.
## How to run: 
### Server
```bash
cargo run -- server 
```
Example:
```bash
cargo run -- server 8080
```
### Client (keyboard input)
```bash
cargo run -- client  
```
Example:
```bash
cargo run -- client 127.0.0.1 8080
cargo run -- client [::1] 8080
cargo run -- client fd00::1 8080 
```
### Client (file input)
```bash
cargo run -- client   
```
Example:
```bash
cargo run -- client 127.0.0.1 8080 messages.txt
cargo run -- client [::1] 8080 messages.txt
cargo run -- client fd00::1 8080 messages.txt
```
On environments: 
for server: 
sudo ip addr add fd00::1/64 dev enp7s0

on clients (first one as example, next can be fd00::3/64, etc.):
sudo ip addr add fd00::2/64 dev enp7s0 

for all need to install rust and enable its environment settings, put enp7s0 up and install build tools for rust:
sudo apt install build-essential (if not already installed)
sudo apt-get install -y net-tools (if not already installed)
sudo apt install unzip
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
sudo ip link set enp7s0 up
scp Lab4.zip with instructions stated in challenges section. 
cargo build (MUST be done before doing cargo run)

On FABRIC, need to SCP a zipped form of Lab4, gunzip to unzip it
then 'cargo build' and then run the above client lines with the loopback
address replaced with the address of the node acting as the server.
My servers did not have the necessary global unicast addresses (only link-local, which are only suitable for point-to-point links) created on all the interfaces so I had to assign them and enable them.
** Challenges **
Realizing that the 'top 5 messages' should be handled as a queue/CircularBuffer. 
Once I realized that, things sort of fell into place on the local side, and getting a small example with 
2 terminals acting as client/server was easy. However, it was much more difficult to actually CONFIGURE the servers correctly, 
especially because the scripts stopped working, btw the Lab4.ipynb setup file and the configure.ipynb file there's configuration
inconsistencies btw the bastion/sliver key and when they should be used. For the SCP command I HAD to use my BASTION key, NOT my 
Sliver key. I also had to use this for scp, where the ipv6 is the enp3s0 global unicast address:  

scp -F /home/fabric/work/fabric_config/ssh_config -i /home/fabric/work/fabric_config/Desktop_Cyrus_Bastion_Key ./Lab4.zip  ubuntu@[2610:1e0:1700:206:f816:3eff:fe7d:dafb]:~/


## Overview

The system consists of a client and server that communicate over UDP sockets. Messages are encrypted before transmission using a simple 
block cipher, and the server maintains a running display of the five most recently received messages. When a client sends a message,
the server responds with its current message history including sender IP and timestamp for each entry.

## Project Structure

```
src/
├── main.rs          - Entry point, CLI mode dispatch
├── client.rs        - UDP client: encrypt, send, receive and display response
├── server.rs        - UDP server: receive, decrypt, maintain history, respond
├── encryption.rs    - Block cipher encryption (from Lab 1)
├── decryption.rs    - Block cipher decryption (from Lab 1)
├── checksum.rs      - One's complement checksum (from Lab 2)
├── headers.rs       - UDP/pseudo header construction (from Lab 2)
├── sender.rs        - Manual datagram sender (from Lab 2/3)
├── receiver.rs      - Manual datagram receiver (from Lab 2/3)
├── file_io.rs       - File read/write utilities
├── utilities.rs     - Shared types, CLI parsing, message serialization
└── unittests.rs     - Unit tests
```

## How It Works

### Encryption

Messages are encrypted using the block cipher from Lab 1. Each 16-bit block of the message has its upper and lower 8-bit halves swapped, then the lower half is XOR'd with a key byte. An 8-byte key file (`key.txt`) is cycled through for each block. Decryption reverses these steps.

### Client

1. Reads a message either from a file or keyboard input
2. Validates the message is under 250 characters and contains no `|` characters
3. Pads to even length if needed, then encrypts in 16-bit blocks
4. Sends the encrypted bytes to the server via UDP
5. Waits up to 5 seconds for a response
6. Displays the server's last five messages with timestamps and sender IPs

### Server

1. Binds to the specified port and enters a receive loop
2. For each incoming packet, decrypts the payload using the same block cipher
3. Decodes the result as UTF-8 and strips null padding bytes
4. Appends the message to a history list, keeping only the most recent 5
5. Prints the current bulletin board to stdout
6. Serializes the history and sends it back to the client

### Message Wire Format

The server response uses a simple pipe-delimited, newline-separated text format:

```
<sender_ip>|<unix_timestamp>|<message_content>\n
```

For example:
```
192.168.0.1|1714500000|hello world
10.0.0.2|1714500050|second message
```

This is parsed on the client side to display the bulletin board.

## Dependencies

- Rust standard library only — no external crates required
- A `key.txt` file containing at least 8 bytes must be present in the working directory for both client and server

## Testing Locally

Open two terminals:

**Terminal 1 — start the server:**
```bash
cargo run -- server 8080
```

**Terminal 2 — run a client:**
```bash
cargo run -- client 127.0.0.1 8080
```

To simulate multiple clients, open additional terminals and run more client instances against the same server.

## Running Unit Tests
```bash
cargo run
```
Set `testing = true` in `main.rs` to enable the built-in unit test suite, then run with any valid mode arguments. Tests cover IP parsing, checksum calculation, header construction, byte conversion utilities, message serialization/deserialization, timestamp sanity, and encrypt/decrypt roundtrip correctness.

## Sources

- Claude AI (https://claude.ai)
- Microsoft GitHub Copilot
- https://www.geeksforgeeks.org/c/socket-programming-cc/
- *Head First C* by David & Dawn Griffiths