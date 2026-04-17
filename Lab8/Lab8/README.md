# P2P File Sharing System

Rust implementation of a centralized P2P file sharing system using a ring topology.

---

## Demo Test Procedure

This section shows the exact steps to demonstrate all required functionality with 1 bootstrap server and 4 peer nodes.

### Setup — run these on every node first

```
sudo ip link set enp7s0 up
python3 generate_files.py
```

Assign addresses (one per node):
```
# Bootstrap node
sudo ip addr add fd00::1/64 dev enp7s0

# Client 1
sudo ip addr add fd00::2/64 dev enp7s0

# Client 2
sudo ip addr add fd00::3/64 dev enp7s0

# Client 3
sudo ip addr add fd00::4/64 dev enp7s0

# Client 4
sudo ip addr add fd00::5/64 dev enp7s0
```

### Step 1 — Start bootstrap server (fd00::1 node)

```
python3 bootstrap.py
```

Note the port it prints, e.g. `Bootstrap server listening on ip: fd00::1, Port: 54321`

### Step 2 — Join peers in order

Each command goes in a separate SSH session. Wait for each peer to print "Joined network successfully" before starting the next one.

```
# Client 1 (fd00::2)
./target/debug/Lab8 fd00::2 enps7s0 5000 fd00::1 <bootstrap_port> 

# Client 2 (fd00::3)
./target/debug/Lab8 fd00::3 enps7s0 5002 fd00::1 <bootstrap_port> 

# Client 3 (fd00::4)
./targetdebug/Lab8  fd00::4 enp7s0 5002 fd00::1 <bootstrap_port>

# Client 4 (fd00::5)
./target/debug/Lab8  fd00::5 enp7s0 5003 fd00::1 <bootstrap_port>
```

After all four peers join, run `display` on each and confirm the ring is correct:
```
Peer fd00::2 — successor: fd00::3, predecessor: fd00::5
Peer fd00::3 — successor: fd00::4, predecessor: fd00::2
Peer fd00::4 — successor: fd00::5, predecessor: fd00::3
Peer fd00::5 — successor: fd00::2, predecessor: fd00::4
```

### Step 3 — Check what files each peer has

Run `display` on each peer and note the filenames in `./own`. You need a filename that exists on Client 4 (fd00::5) but not on Client 1 (fd00::2), and a filename that exists on Client 2 (fd00::3) but not on Client 3 (fd00::4).

### Step 4 — Client 1 queries a file from Client 4

On Client 1 (fd00::2):
```
> query 5 <filename_from_client4>
```

Expected output on Client 1:
```
[Query] Sending query for '<filename>' to successor fd00::3:5001
[Transfer] FILE_FOUND, downloading...
[Transfer] Downloaded '<filename>' saved to ./own
```

Confirm the file appeared:
```
> display
```

The file should now appear in Client 1's `./own` list.

### Step 5 — Client 3 queries a file from Client 2

On Client 3 (fd00::4):
```
> query 5 <filename_from_client2>
```

Expected output on Client 3:
```
[Query] Sending query for '<filename>' to successor fd00::5:5003
[Transfer] FILE_FOUND, downloading...
[Transfer] Downloaded '<filename>' saved to ./own
```

Confirm the file appeared in Client 3's `./own`.

### Step 6 — Client 4 leaves the network

On Client 4 (fd00::5):
```
> leave
```

Expected output:
```
Notified successor and predecessor of departure
Unregistered from bootstrap
Goodbye.
```

After Client 4 leaves, run `display` on Client 1 and Client 3 to confirm the ring repaired itself:
```
Peer fd00::2 — successor: fd00::3, predecessor: fd00::4
Peer fd00::4 — successor: fd00::2, predecessor: fd00::3
```

### Step 7 — Client 1 queries Client 4's file again (should fail)

On Client 1 (fd00::2):
```
> query 5 <filename_that_was_only_on_client4>
```

Expected output — query circulates the ring and finds no one with the file:
```
[Query] Sending query for '<filename>' to successor fd00::3:5001
[Query] Hop count reached 0, dropping query for '<filename>'
```

No FILE_FOUND is received and no download occurs, confirming Client 4 has left the network and its files are no longer reachable.

---

## Architecture

```
Bootstrap Server (Python/UDP) — peer discovery only
        ↓
Peer Node (Rust) — UDP for ring messages, TCP for file transfers
        ↓
Ring: fd00::2 <-> fd00::3 <-> fd00::4 <-> fd00::5 <-> fd00::2
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
cargo build 
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

## FABRIC Setup

**On all nodes — install dependencies:**
```
sudo apt install build-essential python3-pip
pip3 install netifaces
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
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

## CLI Commands

```
query <hop_count> <filename>   Query the ring for a file
display                        Show own IP/port, successor, predecessor, and files in ./own
leave                          Unregister from bootstrap and exit cleanly
```

## Challenges

The most complex part was the ring insertion logic — when a new peer joins it needs to update four pointers across three nodes (its own successor and predecessor, the existing peer's successor, and the new successor's predecessor) using only unreliable UDP messages. Careful ordering of SET_SUCCESSOR and SET_PREDECESSOR messages is required to avoid a window where the ring is broken.

The file transfer port convention of own_port + 1 for TCP avoids needing a separate port negotiation protocol while keeping UDP peer messages and TCP file transfers on separate sockets.

## Sources

- Claude AI (https://claude.ai)
- Microsoft GitHub Copilot
- https://www.geeksforgeeks.org/c/socket-programming-cc/
- Beej's Guide to Network Programming (https://beej.us/guide/bgnet/)