# ESFTP (Extremely Simple File Transfer Protocol) — Concurrent Server

Rust implementation of a concurrent TCP-based file transfer system. The server handles up to 5 simultaneous file transfers, with up to 5 more queued in FIFO order. Connections beyond that receive a SERVER_FULL response. The client connects, proposes a file, and if accepted sends it in 1024 byte chunks.

## Compilation

Make sure Rust and Cargo are installed:
```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

Then build:
```
cargo build
```

## How to run:

### TCP Server
```
cargo run -- tcp_server <port>
```
Example:
```
cargo run -- tcp_server 8080
```

### TCP Client
```
cargo run -- tcp_client <server_ip> <port> <filename>
```
Example:
```
cargo run -- tcp_client ::1 8080 name.txt
cargo run -- tcp_client fd00::1 8080 name.txt
```

## On FABRIC environments:

On server node:
```
sudo ip link set enp7s0 up
sudo ip addr add fd00::1/64 dev enp7s0
```

On client nodes (increment address for each):
```
sudo ip link set enp7s0 up
sudo ip addr add fd00::2/64 dev enp7s0
```

Install dependencies on all nodes:
```
sudo apt install build-essential
sudo apt-get install -y net-tools
sudo apt install unzip
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

SCP the zipped project to each node (must use Bastion key and enp3s0 global unicast address):
```
scp -F /home/fabric/work/fabric_config/ssh_config -i /home/fabric/work/fabric_config/Desktop_Cyrus_Bastion_Key ./Lab6.zip ubuntu@[<enp3s0_ipv6>]:~/
```

Then on each node:
```
unzip Lab6.zip
cd Lab6
cargo build
```

Replace `::1` loopback with the `fd00::` address of the server node when running on FABRIC.

## Overview

The system implements a concurrent TCP file transfer protocol. The server accepts up to 5 simultaneous transfers using a thread per connection model. Additional connections up to 5 are queued in FIFO order using a VecDeque. Connections arriving when both the active slots and queue are full receive a SERVER_FULL response and are dropped immediately. A shared stdin mutex ensures that server operator prompts from multiple threads do not interleave, and each prompt includes the peer address so the operator knows which client they are responding to.

## Project Structure

```
src/
├── main.rs          - Entry point, CLI mode dispatch
├── tcp_client.rs    - TCP client: connect, send file in 1024 byte chunks
├── tcp_server.rs    - TCP server: concurrent accept/reject, threaded transfers
├── utilities.rs     - Shared types and CLI parsing
```

## How It Works

### TCP Client

1. Connects to the server at the given IP and port
2. Sends the filename to the server
3. Waits for server response: ACCEPT, REJECT, or SERVER_FULL
4. If rejected or server is full, closes the connection without sending data
5. If accepted, reads the file in 1024 byte chunks and sends them sequentially
6. Shuts down the write side of the socket to signal end of file to the server
7. Waits for SUCCESS confirmation and informs the user

### TCP Server

1. Binds to the specified port and enters an accept loop
2. For each incoming connection, checks active thread count and queue size
3. If a slot is free, increments the active counter and spawns a thread immediately
4. If all slots are busy but queue has room, pushes the connection to the back of the FIFO queue
5. If both are full, sends SERVER_FULL and drops the connection
6. Each thread handles accept/reject negotiation with the operator, receives the file in chunks, writes to disk, and confirms receipt
7. When a thread finishes it decrements the active counter and checks the queue, popping and spawning the next waiting connection if one exists

### Concurrency Model

The active thread counter and connection queue are both wrapped in `Arc<Mutex<>>` so they can be safely shared across threads. Both locks are acquired together before making any dispatch decision to avoid race conditions. A separate `Arc<Mutex<()>>` stdin lock serializes operator prompts so only one thread reads from stdin at a time.

### Chunk Transfer

Files are transferred in constant 1024 byte chunks. End of file is signaled by shutting down the write side of the TCP socket (`Shutdown::Write`), which causes the server's read to return 0 and exit the receive loop cleanly.

### Wire Protocol

All control messages are newline terminated plain text over the same TCP connection as the file data:

```
client -> server:  "filename.txt\n"
server -> client:  "ACCEPT\n" | "REJECT\n" | "SERVER_FULL\n"
client -> server:  [1024 byte chunks...]
client -> server:  [TCP FIN via Shutdown::Write]
server -> client:  "SUCCESS\n"
```

## Dependencies

Rust standard library only, no external crates required.

## Testing Locally

**Terminal 1 — start the server:**
```
cargo run -- tcp_server 8080
```

**Terminal 2 — run a client:**

On Windows use the compiled binary directly since cargo cannot recompile while the server holds the executable:
```
target\debug\Lab6.exe tcp_client ::1 8080 name.txt
```

## Verification

### Basic transfer

1. Create a test file with some content, e.g. `name.txt`
2. Start the server and run the client
3. At the server prompt type `yes`
4. After transfer verify the saved file matches the original:
```
diff name.txt <saved_filename>
```
A clean diff with no output confirms byte-for-byte accurate transfer.

### Testing concurrency and queue

Small files transfer too fast to fill the slots. Generate a large file first:
```
dd if=/dev/urandom of=bigfile.bin bs=1M count=10
```
This creates a 10MB file filled with random bytes. Then launch 8 or more clients simultaneously:
```
for i in {1..8}; do ./Lab6 tcp_client fd00::1 8080 bigfile.bin & done
```
On the server you should see:
- First 5 connections dispatched immediately to threads
- Connections 6-8 queued with messages like `All slots busy, queuing connection (1/5)`
- As transfers finish, queued connections dequeued automatically
- If you launch more than 10 simultaneously you will see `SERVER_FULL`

## Challenges

The most significant challenge was understanding Rust's concurrency model. Unlike C where you pass raw pointers to threads and manage lifetimes manually, Rust requires you to be explicit about both shared ownership and mutual exclusion separately. `Arc` handles shared ownership across threads via reference counting, and `Mutex` handles exclusive access. Using them together as `Arc<Mutex<T>>` is the standard Rust pattern for shared mutable state across threads, equivalent to a heap-allocated mutex-protected variable in C but with compiler-enforced safety guarantees.

A secondary challenge was the stdin interleaving problem — when multiple threads all try to prompt the server operator simultaneously their output and input interleave in the terminal. This was solved by adding a dedicated stdin mutex that serializes all operator interaction to one thread at a time, and including the peer address in each prompt so the operator knows which client they are responding to.

FABRIC configuration remained a challenge as in previous labs, particularly around key management and manually assigning IPv6 addresses to the experiment network interface.

## Sources

- Claude AI (https://claude.ai)
- Microsoft GitHub Copilot
- https://www.geeksforgeeks.org/c/socket-programming-cc/
- *Head First C* by David & Dawn Griffiths