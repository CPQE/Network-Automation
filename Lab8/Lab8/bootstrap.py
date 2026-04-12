import socket
import threading
import netifaces
import random
import time

# Format: (ip_with_interface, port)
registered_peers = []

INTERFACE = 'enp7s0'
PORT = 0  # Use 0 to let OS assign an available port
LOG_FILE = "log.txt"

def log_message(message):
    timestamp = time.strftime("%Y-%m-%d %H:%M:%S", time.localtime())
    full_message = f"[{timestamp}] {message}"
    print(full_message)
    with open(LOG_FILE, 'a') as f:
        f.write(full_message + '\n')

def get_ipv6_address(interface):
    try:
        for link in netifaces.ifaddresses(interface)[netifaces.AF_INET6]:
            ip = link['addr']
            if '%' in ip:
                ip = ip.split('%')[0]
            return ip
    except (ValueError, KeyError):
        return None

def list_peers():
    return ", ".join([f"{ip}:{port}" for ip, port in registered_peers]) or "<no peers>"

def handle_packet(data, addr, server_socket):
    global registered_peers
    data = data.decode().strip()
    log_message(f"Received from {addr}: {data}")

    parts = data.split()
    response = ""

    if not parts:
        response = "ERROR 1 Empty command"

    else:
        cmd = parts[0]

        if cmd == "REG" and len(parts) == 3:
            peer = (parts[1], parts[2])
            if peer in registered_peers:
                response = "ERROR 2 Peer already exists"
            else:
                registered_peers.append(peer)
                other_peers = [p for p in registered_peers if p != peer]
                if not other_peers:
                    response = "REGOK 0"
                else:
                    selected = random.choice(other_peers)
                    response = f"REGOK 1 {selected[0]} {selected[1]}"

        elif cmd == "REG" and len(parts) != 3:
            response = "ERROR 3 Invalid REG format"

        elif cmd == "UNREG" and len(parts) == 3:
            peer = (parts[1], parts[2])
            if peer in registered_peers:
                registered_peers.remove(peer)
                response = "UNREGOK 0"
            else:
                response = "ERROR 4 Peer not found for UNREG"

        elif cmd == "UNREG" and len(parts) != 3:
            response = "ERROR 5 Invalid UNREG format"

        else:
            response = "ERROR 6 Unknown command"

    log_message(f"Sent: {response}")
    log_message(f"Current peers: {list_peers()}")
    server_socket.sendto(response.encode(), addr)

def run_server():
    ip = get_ipv6_address(INTERFACE)
    if ip is None:
        log_message(f"Could not determine IPv6 address for interface {INTERFACE}.")
        return

    with socket.socket(socket.AF_INET6, socket.SOCK_DGRAM) as server_socket:
        interface_index = socket.if_nametoindex(INTERFACE)
        server_socket.bind((ip, PORT, 0, interface_index))  # Bind to an available port
        actual_port = server_socket.getsockname()[1]
        log_message(f"Bootstrap server listening on ip: {ip}, interface: {INTERFACE}, Port: {actual_port} (UDP)")

        while True:
            data, addr = server_socket.recvfrom(1024)
            threading.Thread(target=handle_packet, args=(data, addr, server_socket)).start()

if __name__ == '__main__':
    run_server()
