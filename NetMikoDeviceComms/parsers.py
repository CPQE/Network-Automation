"""
Parsers for Cisco IOS command output (concise, resilient).
Provides functions used by inventory_collector.py.

Each function accepts raw command output (str) and returns
a compact Python structure (dict/list) suitable for JSON serialization.
"""

import re
from typing import Dict, List


def parse_show_version(text: str) -> Dict:
    if not text:
        return {}
    info = {}
    for line in text.splitlines():
        if " uptime is " in line and " " in line:
            parts = line.split()
            info.setdefault("hostname", parts[0])
            info.setdefault("uptime", " ".join(parts[3:]))
        if "Cisco IOS Software" in line or ("Version" in line and "IOS" in line):
            info.setdefault("os_version", line.strip())
        m = re.search(r"Model number\s*:\s*(\S+)", line)
        if m:
            info.setdefault("model", m.group(1))
        m2 = re.search(r"Processor board ID\s*:?\s*(\S+)", line)
        if m2:
            info.setdefault("serial_number", m2.group(1))
        if "System image file is" in line:
            info.setdefault("system_image", line.split("System image file is", 1)[1].strip().strip('"'))
    return info


def parse_ip_interface_brief(text: str) -> List[Dict]:
    if not text:
        return []
    lines = [l for l in text.splitlines() if l.strip()]
    result = []
    header_found = False
    for line in lines:
        if not header_found and re.search(r"Interface\s+IP-?Address", line, re.I):
            header_found = True
            continue
        if not header_found:
            continue
        cols = re.split(r"\s+", line.strip())
        if len(cols) >= 6:
            result.append({"name": cols[0], "ip": cols[1], "ok": cols[2], "method": cols[3], "status": cols[4], "protocol": cols[5]})
        else:
            # best-effort fallback
            parts = line.split()
            if parts:
                name = parts[0]
                ip = parts[1] if len(parts) > 1 else "unassigned"
                status = parts[-2] if len(parts) > 2 else ""
                proto = parts[-1] if len(parts) > 1 else ""
                result.append({"name": name, "ip": ip, "status": status, "protocol": proto})
    return result


def parse_interfaces_status(text: str) -> Dict[str, Dict]:
    if not text:
        return {}
    lines = [l for l in text.splitlines() if l.strip()]
    # split columns by 2+ spaces
    out = {}
    # attempt to find header, else parse heuristically
    header_idx = next((i for i, l in enumerate(lines) if re.search(r"^Port\s+Name\s+Status\s+Vlan", l, re.I)), None)
    start = header_idx + 1 if header_idx is not None else 0
    for line in lines[start:]:
        cols = re.split(r"\s{2,}", line.strip())
        if not cols:
            continue
        name = cols[0].split()[0]
        entry = {"name": name}
        if header_idx is not None:
            entry.update({
                "description": cols[1] if len(cols) > 1 else "",
                "status": cols[2] if len(cols) > 2 else "",
                "vlan": cols[3] if len(cols) > 3 else "",
                "duplex": cols[4] if len(cols) > 4 else "",
                "speed": cols[5] if len(cols) > 5 else "",
                "type": cols[6] if len(cols) > 6 else "",
            })
        else:
            entry["status"] = cols[1] if len(cols) > 1 else ""
        out[name] = entry
    return out


def parse_interface_counters(text: str) -> Dict[str, Dict]:
    if not text:
        return {}
    out = {}
    for line in text.splitlines():
        line = line.strip()
        if not line:
            continue
        # try to find "Interface  input errors  output errors" style lines
        m = re.match(r"^(\S+)\s+(\d+)\s+(\d+)", line)
        if m:
            iface, in_err, out_err = m.group(1), int(m.group(2)), int(m.group(3))
            out.setdefault(iface, {})["input_errors"] = in_err
            out[iface]["output_errors"] = out_err
    return out


def parse_vlan_brief(text: str) -> List[Dict]:
    if not text:
        return []
    vlans = []
    lines = text.splitlines()
    header_found = next((i for i, l in enumerate(lines) if re.search(r"^VLAN\s+Name\s+Status", l, re.I)), None)
    if header_found is None:
        return vlans
    for line in lines[header_found + 1 :]:
        if not line.strip():
            continue
        cols = re.split(r"\s{2,}", line.strip())
        if not cols:
            continue
        vid = cols[0].split()[0]
        name = cols[1] if len(cols) > 1 else ""
        status = cols[2] if len(cols) > 2 else ""
        ports = [p.strip() for p in cols[3].split(",")] if len(cols) > 3 and cols[3].strip() else []
        vlans.append({"id": vid, "name": name, "status": status, "ports": ports})
    return vlans


def parse_mac_table(text: str) -> List[Dict]:
    if not text:
        return []
    entries = []
    lines = text.splitlines()
    header_found = next((i for i, l in enumerate(lines) if re.search(r"^Vlan\s+Mac Address\s+Type", l, re.I)), None)
    if header_found is None:
        return entries
    for line in lines[header_found + 1 :]:
        if not line.strip() or line.strip().startswith("---"):
            continue
        cols = re.split(r"\s{2,}", line.strip())
        if len(cols) >= 3:
            vlan = cols[0].split()[0]
            mac = cols[1]
            type_ = cols[2]
            ports = [p.strip() for p in cols[3].split(",")] if len(cols) > 3 else []
            entries.append({"vlan": vlan, "mac": mac, "type": type_, "ports": ports})
    return entries


def parse_arp(text: str) -> List[Dict]:
    if not text:
        return []
    arps = []
    lines = text.splitlines()
    header_found = next((i for i, l in enumerate(lines) if re.search(r"^Protocol\s+Address\s+Age", l, re.I)), None)
    if header_found is None:
        # try lines like "Internet  10.0.0.1  2   5254.abcd.1234  ARPA  GigabitEthernet0/0"
        for line in lines:
            cols = re.split(r"\s{2,}", line.strip())
            if len(cols) >= 5:
                arps.append({"ip": cols[1], "age": cols[2], "mac": cols[3], "type": cols[4], "interface": cols[5] if len(cols) > 5 else ""})
        return arps
    for line in lines[header_found + 1 :]:
        if not line.strip():
            continue
        cols = re.split(r"\s{2,}", line.strip())
        if len(cols) >= 5:
            arps.append({"ip": cols[1], "age": cols[2], "mac": cols[3], "type": cols[4], "interface": cols[5] if len(cols) > 5 else ""})
    return arps


def parse_ip_route(text: str) -> List[Dict]:
    if not text:
        return []
    routes = []
    for line in text.splitlines():
        line = line.strip()
        if not line or line.lower().startswith("gateway of last resort"):
            continue
        m = re.match(r"^([A-Z\*\+\s\/]+)\s+(\d+\.\d+\.\d+\.\d+\/\d+)\s+(.*)$", line)
        if m:
            code = m.group(1).strip()
            prefix = m.group(2).strip()
            detail = m.group(3).strip()
            via = None
            iface = None
            mv = re.search(r"via\s+(\d+\.\d+\.\d+\.\d+)", detail)
            if mv:
                via = mv.group(1)
            mi = re.search(r",\s*(\S+)$", detail)
            if mi:
                iface = mi.group(1)
            routes.append({"code": code, "prefix": prefix, "via": via, "interface": iface, "detail": detail})
        else:
            # fallback: store raw line
            routes.append({"raw": line})
    return routes


def parse_spanning_tree_summary(text: str) -> Dict:
    if not text:
        return {}
    info = {}
    for line in text.splitlines():
        line = line.strip()
        m = re.search(r"Total number of VLANs:\s*(\d+)", line, re.I)
        if m:
            info["total_vlans"] = int(m.group(1))
        m2 = re.search(r"Hello Time\s+(\d+)\s+sec", line)
        if m2:
            info["hello_time"] = int(m2.group(1))
        m3 = re.search(r"Max Age\s+(\d+)\s+sec", line)
        if m3:
            info["max_age"] = int(m3.group(1))
        m4 = re.search(r"Forward Delay\s+(\d+)\s+sec", line)
        if m4:
            info["forward_delay"] = int(m4.group(1))
    return info


def parse_ip_ospf_neighbor(text: str) -> List[Dict]:
    if not text:
        return []
    neighbors = []
    lines = text.splitlines()
    header_found = next((i for i, l in enumerate(lines) if re.search(r"^Neighbor ID\s+Pri", l, re.I)), None)
    if header_found is None:
        # try simple parsing of any IP-like occurrences
        for line in lines:
            cols = re.split(r"\s+", line.strip())
            if len(cols) >= 6 and re.match(r"\d+\.\d+\.\d+\.\d+", cols[0]):
                neighbors.append({"neighbor_id": cols[0], "priority": cols[1], "state": cols[2], "dead_time": cols[3], "address": cols[4], "interface": cols[5]})
        return neighbors
    for line in lines[header_found + 1 :]:
        if not line.strip():
            continue
        cols = re.split(r"\s+", line.strip())
        if len(cols) >= 6:
            neighbors.append({"neighbor_id": cols[0], "priority": cols[1], "state": cols[2], "dead_time": cols[3], "address": cols[4], "interface": cols[5]})
    return neighbors