# ESFTP (Extremely Simple File Transfer Protocol)

Rust implementation of a TCP-based file transfer system. A client connects to a server, proposes a file transfer, and if accepted sends the file in 1024 byte chunks. The server saves the file locally and confirms receipt.

## How to run:

### TCP Server
```bash
cargo run -- tcp_server <port>
```
Example:
```bash
cargo run -- tcp_server 8080
```

### TCP Client
```bash
cargo run -- tcp_client <server_ip> <port> <filename>
```
Example:
```bash
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

SCP or upload the zipped project to each node (must use Bastion key and enp3s0 global unicast address):
```
scp -F /home/fabric/work/fabric_config/ssh_config -i /home/fabric/work/fabric_config/Desktop_Cyrus_Bastion_Key ./Lab5.zip ubuntu@[<enp3s0_ipv6>]:~/
```

Then on each node:
```
unzip Lab5.zip
cd Lab5
cargo build
```

Replace `::1` loopback with the `fd00::` address of the server node when running on FABRIC.

## Overview

The system implements a minimal TCP file transfer protocol. The client initiates a connection and proposes a file by name. The server operator can accept or reject the transfer. If accepted, the file is sent in 1024 byte chunks and reassembled on the server side. The server confirms successful receipt and then waits for the next client.

## Project Structure

```
src/
├── main.rs          - Entry point, CLI mode dispatch
├── tcp_client.rs    - TCP client: connect, send file in 1024 byte chunks
├── tcp_server.rs    - TCP server: accept/reject files, receive chunks, save to disk
├── utilities.rs     - Shared types and CLI parsing
```

## How It Works

### TCP Client

1. Connects to the server at the given IP and port
2. Sends the filename to the server
3. Waits for server response: `ACCEPT` or `REJECT`
4. If rejected, closes the connection without sending any data
5. If accepted, reads the file in 1024 byte chunks and sends them sequentially
6. Shuts down the write side of the socket to signal end of file
7. Waits for `SUCCESS` confirmation from the server and informs the user

### TCP Server

1. Binds to the specified port and waits for incoming connections
2. Receives the filename from the client and displays it to the server operator
3. Prompts the operator to accept or reject the transfer
4. If rejected, sends `REJECT` and closes the connection
5. If accepted, prompts for a local save name, sends `ACCEPT`, and receives chunks
6. Writes each chunk to disk as it arrives
7. Sends `SUCCESS` confirmation to the client and waits for the next connection

### Chunk Transfer

Files are transferred in constant 1024 byte chunks. The end of the file is signaled by shutting down the write side of the TCP socket (`Shutdown::Write`), which causes the server's `read` to return 0 bytes and exit the receive loop cleanly.

## Dependencies

- Rust standard library only, no external crates required

## Testing Locally

Open two terminals:

**Terminal 1 — start the server:**
```bash
cargo run -- tcp_server 8080
```

**Terminal 2 — run the client:**
```bash
target\debug\Lab5.exe tcp_client ::1 8080 name.txt
```

On Windows use the compiled binary directly in the second terminal since cargo cannot recompile while the server process is holding the executable.

To test multiple clients, run the client from additional terminals sequentially — the server loops back to waiting after each completed transfer.

## Verification

1. Create a test file on the client node, e.g. `name.txt` with some content
2. Start the server, run the client pointing at the server
3. Server prompts to accept — type `yes` and provide a save name
4. After transfer, confirm the saved file on the server matches the original:
```
diff name.txt <saved_file>
```
A clean diff with no output confirms the transfer was byte-for-byte accurate.

## Challenges

Configuring FABRIC nodes correctly was the most difficult part. Between the Lab5.ipynb setup file and the configure.ipynb file there are configuration inconsistencies between the bastion and sliver keys. For SCP the Bastion key must be used, not the Sliver key, and the enp3s0 global unicast address must be used. The experiment interface enp7s0 comes up without any global unicast address assigned, so `fd00::` addresses must be manually assigned on each node before the client and server can communicate over the experiment network.

## Sources

- Claude AI (https://claude.ai)
- Microsoft GitHub Copilot
- https://www.geeksforgeeks.org/c/socket-programming-cc/
- *Head First C* by David & Dawn Griffiths
