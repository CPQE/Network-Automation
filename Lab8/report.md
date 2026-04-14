# Lab 8 Simple Report

## Work Completed

Working individually.
- Bootstrap registration and unregistration (REG/UNREG over UDP)
- Peer ring join/leave logic (JOIN, JOINOK, SET_SUCCESSOR, SET_PREDECESSOR)
- UDP listener and message dispatcher
- FILE_QUERY forwarding with hop count
- FILE_FOUND notification and TCP file download
- CLI for user interaction (query, display, leave)
- Ansible automation for FABRIC node setup and deployment

## Tools Used

- Claude AI (https://claude.ai)
- Microsoft GitHub Copilot
- Ansible (node setup and deployment automation)
- Rust standard library documentation (https://doc.rust-lang.org)

## Tests Performed

### Test 1 — Peer Join and Ring Formation

Started the bootstrap server on fd00::1, then joined four peers in order: fd00::2 (port 5000), fd00::3 (port 5001), fd00::4 (port 5002), fd00::5 (port 5003).

**Observed behavior:** Bootstrap server logged each REG message and responded with REGOK 0 for the first peer and REGOK 1 with an existing peer's address for subsequent peers. Each peer printed its successor and predecessor after joining.

**Why it is correct:** After all four peers joined the ring should form a circular structure where each peer's successor is the next peer and predecessor is the previous one. Running `display` on each peer confirmed:
```
fd00::2 — successor: fd00::3, predecessor: fd00::5
fd00::3 — successor: fd00::4, predecessor: fd00::2
fd00::4 — successor: fd00::5, predecessor: fd00::3
fd00::5 — successor: fd00::2, predecessor: fd00::4
```
This matches the expected ring topology.

### Test 2 — File Query and Download (Client 1 queries Client 4's file)

 Ran `display` on fd00::2 and fd00::5 to identify a file that existed on fd00::5 but not fd00::2. Then on fd00::2 ran:
```
query 5 <filename_from_fd00::5>
```

**Observed behavior:** The FILE_QUERY message was forwarded through the ring from fd00::2 to fd00::3 to fd00::4 to fd00::5. fd00::5 found the file locally and sent FILE_FOUND directly back to fd00::2. fd00::2 then opened a TCP connection to fd00::5 and downloaded the file into its ./own directory.

**Why it is correct:** Running `display` on fd00::2 after the query shoId the file now appearing in its ./own list, confirming the download completed successfully. The hop count decremented correctly at each peer.

### Test 3 — File Query and Download (Client 3 queries Client 2's file)

 On fd00::4 ran:
```
query 5 <filename_from_fd00::3>
```

**Observed behavior:** Query forwarded from fd00::4 to fd00::5 to fd00::2 to fd00::3. fd00::3 found the file and sent FILE_FOUND back to fd00::4 which downloaded it successfully.

**Why it is correct:** File appeared in fd00::4's ./own directory after the query confirming correct ring traversal and direct peer-to-peer TCP download.

### Test 4 — Peer Leave and Ring Repair

 On fd00::5 ran:
```
leave
```

**Observed behavior:** fd00::5 sent SET_PREDECESSOR to fd00::2 with fd00::4's address, and SET_SUCCESSOR to fd00::4 with fd00::2's address. Bootstrap server logged the UNREG message and responded UNREGOK 0.

**Why it is correct:** Running `display` on fd00::2 and fd00::4 after fd00::5 left confirmed the ring repaired correctly:
```
fd00::2 — successor: fd00::3, predecessor: fd00::4
fd00::4 — successor: fd00::2, predecessor: fd00::3
```
fd00::5 is no longer referenced by any peer.

### Test 5 — Query After Peer Leave (Expected Failure)

On fd00::2 queried for a file that was only on fd00::5 after it had left:
```
query 5 <filename_that_was_only_on_fd00::5>
```

**Observed behavior:** The query circulated through the remaining three peers (fd00::2, fd00::3, fd00::4) and the hop count reached 0 without finding the file. No FILE_FOUND was received and no download occurred.

**Why it is correct:** fd00::5 has left the network and unregistered from the bootstrap. Its files are no longer reachable. The query correctly expired after traversing all remaining peers, confirming the ring is stable with three nodes and unreachable files fail gracefully rather than causing a crash or infinite loop.