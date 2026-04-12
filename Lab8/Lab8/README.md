# P2P File Sharing System

Rust implementation of a centralized P2P file sharing system using a ring topology. Peers register with a bootstrap server for initial discovery, maintain successor and predecessor links in a ring, and can query and download files from other peers.

## Architecture

```
Bootstrap Server (Python/UDP) — peer discovery only
        ↓
Peer Node (Rust) — UDP for ring messages, TCP for file transfers
        ↓
Ring: Peer A <-> Peer B <-> Peer C <-> Peer A
```

Each peer runs three concurrent components:
- UDP listener — handles JOIN, SET_SUCCESSOR, SET_PREDECESSOR, FILE_QUERY, FILE_FOUND
- TCP file server — serves files to peers that want to download
- CLI — accepts user commands (query, display, leave)

## Message Protocol

### Bootstrap messages (UDP)
```
REG ip%interface port          -> REGOK 0 | REGOK 1 ip%interface port
UNREG ip%interface port        -> UNREGOK 0
```

### Peer-to-peer messages (UDP)
```
JOIN ip%interface port         -> JOINOK ip%interface port
SET_SUCCESSOR ip%interface port
SET_PREDECESSOR ip%interface port
FILE_QUERY hop_count filename source_ip%interface source_port
FILE_FOUND filename ip%interface port
```

### File transfer (TCP on own_port + 1)
```
client -> server: "filename\n"
server -> client: "OK\n" | "ERROR message\n"
client <- server: [file chunks...]
```

## Compilation

Make sure Rust is installed:
```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

Build:
```
cargo build --release
```

Bootstrap server requires Python with netifaces:
```
pip3 install netifaces
```

## Project Structure

```
src/
├── main.rs        - Entry point, argument parsing, thread spawning
├── bootstrap.rs   - REG/UNREG messages to bootstrap server
├── peer.rs        - PeerState struct, JOIN/leave ring logic
├── listener.rs    - UDP listener, message dispatch
├── query.rs       - FILE_QUERY forwarding, FILE_FOUND notification
├── transfer.rs    - TCP file server and file download
├── files.rs       - Local file utilities
└── cli.rs         - User prompt loop
```

## Running Locally

You need at least 3 terminals.

**Terminal 1 — Bootstrap server:**
```
python3 bootstrap.py
```
Note the port it prints — you will need it for the peers.

**Terminal 2 — Generate files and start Peer A:**
```
python3 generate_files.py
cargo run --release -- 127.0.0.1 lo 5000 127.0.0.1 <bootstrap_port>
```

**Terminal 3 — Generate files and start Peer B:**
```
python3 generate_files.py
cargo run --release -- 127.0.0.1 lo 5001 127.0.0.1 <bootstrap_port>
```

**Terminal 4 — Generate files and start Peer C:**
```
python3 generate_files.py
cargo run --release -- 127.0.0.1 lo 5002 127.0.0.1 <bootstrap_port>
```

## Running on FABRIC

**On all nodes — install dependencies:**
```
sudo apt install build-essential python3-pip
pip3 install netifaces
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

**Assign experiment interface addresses:**
```
# Node 1 (bootstrap + Peer A)
sudo ip link set enp7s0 up
sudo ip addr add fd00::1/64 dev enp7s0

# Node 2 (Peer B)
sudo ip link set enp7s0 up
sudo ip addr add fd00::2/64 dev enp7s0

# Node 3 (Peer C)
sudo ip link set enp7s0 up
sudo ip addr add fd00::3/64 dev enp7s0
```

**SCP project to each node:**
```
scp -F /home/fabric/work/fabric_config/ssh_config \
    -i /home/fabric/work/fabric_config/Desktop_Cyrus_Bastion_Key \
    ./Lab8.zip ubuntu@[<enp3s0_ipv6>]:~/
```

**On each node — unzip and build:**
```
unzip Lab8.zip
cd Lab8
cargo build --release
python3 generate_files.py
```

**Start bootstrap on one node:**
```
python3 bootstrap.py
```
Note the port printed in the output.

**Start peers on each node:**
```
# Node 1 (fd00::1)
./target/release/peer fd00::1 enp7s0 5000 fd00::1 <bootstrap_port>

# Node 2 (fd00::2)
./target/release/peer fd00::2 enp7s0 5001 fd00::1 <bootstrap_port>

# Node 3 (fd00::3)
./target/release/peer fd00::3 enp7s0 5002 fd00::1 <bootstrap_port>
```

## CLI Commands

Once a peer is running, the following commands are available:

```
query <hop_count> <filename>   Query the ring for a file
display                        Show own IP/port, successor, predecessor, and files in ./own
leave                          Unregister from bootstrap and exit cleanly
```

Example session:
```
> display
=== Peer State ===
  Own:         fd00::1%enp7s0:5000
  Successor:   fd00::2%enp7s0:5001
  Predecessor: fd00::3%enp7s0:5002
==================
Files in ./own:
  network_peer_server1.txt
  data_packet_server1.txt

> query 3 network_peer_server2.txt
[Query] Sending query for 'network_peer_server2.txt' to successor fd00::2:5001
[Transfer] FILE_FOUND, downloading...
[Transfer] Downloaded 'network_peer_server2.txt' saved to ./own

> leave
Notified successor and predecessor of departure
Unregistered from bootstrap
Goodbye.
```

## Verification

**Peer join:** After each peer joins run `display` and confirm successor and predecessor are correct per the ring order.

**File query:** Query for a file that exists on another peer. Confirm it appears in `./own` after download:
```
ls ./own
```

**Peer leave:** After a peer leaves run `display` on remaining peers and confirm their successor and predecessor updated correctly to skip the departed peer.

## Challenges

The most complex part was the ring insertion logic — when a new peer joins it needs to update four pointers across three nodes (its own successor and predecessor, the existing peer's successor, and the new successor's predecessor) using only unreliable UDP messages. Careful ordering of SET_SUCCESSOR and SET_PREDECESSOR messages is required to avoid a window where the ring is broken.

The file transfer port convention of own_port + 1 for TCP avoids needing a separate port negotiation protocol while keeping UDP peer messages and TCP file transfers on separate sockets.

## Sources

- Claude AI (https://claude.ai)
- Microsoft GitHub Copilot
- https://www.geeksforgeeks.org/c/socket-programming-cc/
- Beej's Guide to Network Programming (https://beej.us/guide/bgnet/)