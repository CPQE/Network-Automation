"""
Inventory & Baseline Collector (Netmiko-based)
Target devices: Cisco Catalyst 3560 (switch) and Cisco 1921/1920-series (router)

This module provides standalone functions (no external framework) to:
- Connect to a device via Netmiko
- Disable paging and run a list of show commands
- Collect running/startup configs and key show outputs
- Compute checksums and save raw outputs to disk
- Produce a JSON snapshot (metadata + parsed/raw outputs)
- Produce a unified textual diff between two config snapshots

Notes:
- Credentials must be provided by the caller (do NOT hardcode).
- Storage is file-system based by default; adapt save functions to S3/Git/etc.
- Parsing included is intentionally lightweight; replace/adapt with richer parsers as needed.
"""

from netmiko import ConnectHandler, NetmikoAuthenticationException, NetmikoTimeoutException
import os
import json
import hashlib
import difflib
import logging
from datetime import datetime, timezone
from pathlib import Path
from typing import Dict, List, Tuple, Optional

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger("inventory_collector")


# -------------------------
# Device connection helpers
# -------------------------
def make_device_params(
    host: str,
    username: str,
    password: str,
    device_type: str = "cisco_ios",
    secret: Optional[str] = None,
    port: int = 22,
) -> Dict:
    """
    Build a dict compatible with Netmiko ConnectHandler kwargs.
    device_type examples: 'cisco_ios' for IOS devices (3560/1921).
    """
    params = {
        "device_type": device_type,
        "host": host,
        "username": username,
        "password": password,
        "port": port,
        "fast_cli": False,  # safer for some devices; adjust if desired
    }
    if secret:
        params["secret"] = secret
    return params


def connect_device(device_params: Dict, enter_enable: bool = True, timeout: int = 30):
    """
    Connects to a device and returns the Netmiko connection object.
    Caller is responsible for closing the connection (conn.disconnect()).
    """
    try:
        conn = ConnectHandler(**device_params)
        conn.session_timeout = timeout
        if enter_enable and device_params.get("secret"):
            try:
                conn.enable()
            except Exception:
                # Some devices don't need enable; continue
                logger.debug("Enable not required or failed: continuing without enable")
        return conn
    except (NetmikoTimeoutException, NetmikoAuthenticationException) as exc:
        logger.error(f"Failed to connect to {device_params.get('host')}: {exc}")
        raise


def disable_paging(conn) -> None:
    """
    Disable paging on Cisco IOS devices so 'show' outputs are full.
    """
    try:
        conn.send_command("terminal length 0", expect_string=r"#|\$")
    except Exception as exc:
        logger.debug(f"Failed to set terminal length 0: {exc}")


# -------------------------
# Collection functions
# -------------------------
def run_commands(conn, commands: List[str], delay_factor: float = 1.0, timeout: int = 60) -> Dict[str, str]:
    """
    Run a list of commands and return a mapping command -> output.
    Uses send_command; catches exceptions and inserts error messages.
    """
    outputs = {}
    for cmd in commands:
        try:
            out = conn.send_command(cmd, delay_factor=delay_factor, timeout=timeout, strip_prompt=False, strip_command=False)
            outputs[cmd] = out
        except Exception as exc:
            logger.warning(f"Command '{cmd}' failed: {exc}")
            outputs[cmd] = f"<ERROR: {exc}>"
    return outputs


def collect_running_config(conn) -> str:
    """
    Collect the running-config text. For large configs Netmiko will return the full buffered output if paging is disabled.
    """
    try:
        return conn.send_command("show running-config", expect_string=r"#|\$", delay_factor=2, strip_prompt=False, strip_command=False)
    except Exception as exc:
        logger.error(f"Failed to collect running-config: {exc}")
        return f"<ERROR: {exc}>"


def collect_startup_config(conn) -> str:
    """
    Collect the startup-config text. On some platforms permission/timeouts might occur.
    """
    try:
        return conn.send_command("show startup-config", expect_string=r"#|\$", delay_factor=2, timeout=60, strip_prompt=False, strip_command=False)
    except Exception as exc:
        logger.warning(f"Failed to collect startup-config: {exc}")
        return f"<ERROR: {exc}>"


# -------------------------
# Basic parsers (lightweight)
# -------------------------
def parse_show_version(text: str) -> Dict:
    """
    Minimal parsing of 'show version' to capture hostname, model, ios version, and uptime.
    This parser is intentionally simple and relies on common IOS output patterns.
    """
    info = {}
    lines = text.splitlines()
    for line in lines:
        if " uptime is " in line and "router" in line.lower() or "switch" in line.lower():
            # e.g., Router1 uptime is 2 weeks, 3 days, 1 hour, 27 minutes
            parts = line.split()
            info.setdefault("hostname", parts[0])
            info.setdefault("uptime", " ".join(parts[3:]))  # best-effort
        if "Cisco IOS Software" in line or "IOS" in line and "Version" in line:
            info.setdefault("os_version", line.strip())
        if "System image file is" in line:
            info.setdefault("system_image", line.split("System image file is", 1)[1].strip().strip('"'))
        if "bytes of memory" in line and ("processor board" in line.lower() or "processor" in line.lower()):
            # attempt to find model on nearby lines; simpler to capture 'Model number' below
            pass
        if "Model number" in line:
            info.setdefault("model", line.split("Model number", 1)[1].strip())
        if "Processor board ID" in line or "Processor board ID" in line:
            info.setdefault("serial_number", line.split()[-1].strip())
    return info


# -------------------------
# Utilities
# -------------------------
def sha256_text(text: str) -> str:
    """Return SHA256 hex digest for given text."""
    h = hashlib.sha256()
    if isinstance(text, str):
        text = text.encode("utf-8", errors="ignore")
    h.update(text)
    return h.hexdigest()


def save_raw_outputs(base_path: Path, device_id: str, outputs: Dict[str, str]) -> Dict[str, str]:
    """
    Save each command output to a file under base_path/device_id/raw/<sanitized_filename>.
    Returns mapping command -> saved_filepath (string).
    """
    saved = {}
    raw_dir = base_path / device_id / "raw"
    raw_dir.mkdir(parents=True, exist_ok=True)
    for cmd, out in outputs.items():
        # create a safe filename
        safe = cmd.strip().replace(" ", "_").replace("/", "_").replace("|", "_").replace(">", "_")[:120]
        filename = raw_dir / f"{safe}.txt"
        with open(filename, "w", encoding="utf-8") as fh:
            fh.write(out)
        saved[cmd] = str(filename)
    return saved


def save_text_file(path: Path, content: str) -> str:
    path.parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as fh:
        fh.write(content)
    return str(path)


def save_snapshot(base_path: Path, device_id: str, snapshot: Dict) -> str:
    """
    Save the structured JSON snapshot to base_path/device_id/snapshots/<timestamp>.json
    Returns the path to the saved snapshot.
    """
    snaps_dir = base_path / device_id / "snapshots"
    snaps_dir.mkdir(parents=True, exist_ok=True)
    ts = snapshot.get("collection_timestamp", datetime.now(timezone.utc).isoformat()).replace(":", "-")
    filename = snaps_dir / f"{ts}.json"
    with open(filename, "w", encoding="utf-8") as fh:
        json.dump(snapshot, fh, indent=2)
    return str(filename)


def load_snapshot(path: str) -> Dict:
    with open(path, "r", encoding="utf-8") as fh:
        return json.load(fh)


def unified_diff(a: str, b: str, fromfile: str = "old", tofile: str = "new") -> str:
    """
    Produce a unified diff between two texts (string) and return as a single string.
    """
    a_lines = a.splitlines(keepends=True)
    b_lines = b.splitlines(keepends=True)
    diff = difflib.unified_diff(a_lines, b_lines, fromfile=fromfile, tofile=tofile, lineterm="")
    return "".join(diff)


# -------------------------
# High-level orchestration
# -------------------------
DEFAULT_COMMANDS = [
    "show version",
    "show inventory",
    "show running-config",
    "show startup-config",
    "show ip interface brief",
    "show interfaces status",
    "show interfaces counters errors",
    "show mac address-table",
    "show vlan brief",
    "show ip route",
    "show arp",
    "show logging",
    "show cdp neighbors detail",
    "show lldp neighbors detail",
    "show processes cpu history",
    "show spanning-tree summary",
    "show environment",
]


def collect_device_snapshot(
    device_params: Dict,
    save_base: str,
    commands: Optional[List[str]] = None,
    collector_name: str = "inventory_collector_v1",
) -> Dict:
    """
    High-level function to collect a snapshot for a single device and persist raw outputs + structured snapshot JSON.

    Returns the snapshot dict (structured) after saving files to disk.
    """
    host = device_params.get("host") or device_params.get("ip") or "unknown"
    device_id = host.replace(":", "_")
    base_path = Path(save_base)
    commands = commands or DEFAULT_COMMANDS

    conn = None
    try:
        conn = connect_device(device_params)
        disable_paging(conn)

        # Collect outputs
        outputs = run_commands(conn, commands)
        # Ensure we collect running/startup config explicitly as full entries (they are also in outputs)
        running = outputs.get("show running-config") or collect_running_config(conn)
        startup = outputs.get("show startup-config") or collect_startup_config(conn)

        # Basic facts
        version_text = outputs.get("show version") or ""
        facts = parse_show_version(version_text)
        facts.update(
            {
                "mgmt_ip": device_params.get("host"),
                "collector": collector_name,
                "collection_timestamp": datetime.now(timezone.utc).isoformat(),
            }
        )

        # Compute checksums
        running_checksum = sha256_text(running)
        startup_checksum = sha256_text(startup)

        # Save raw outputs to files
        raw_map = save_raw_outputs(base_path, device_id, outputs)
        # also save explicit running/startup config files (clear filenames)
        rc_path = Path(base_path) / device_id / "raw" / "running-config.txt"
        sc_path = Path(base_path) / device_id / "raw" / "startup-config.txt"
        save_text_file(rc_path, running)
        save_text_file(sc_path, startup)

        # Prepare structured snapshot
        snapshot = {
            "device": {
                "hostname": facts.get("hostname"),
                "mgmt_ip": facts.get("mgmt_ip"),
                "vendor": "Cisco",
                "model": facts.get("model"),
                "os_version": facts.get("os_version"),
                "serial_number": facts.get("serial_number"),
                "uptime": facts.get("uptime"),
            },
            "metadata": {
                "collection_timestamp": facts.get("collection_timestamp"),
                "collector": collector_name,
                "raw_outputs": raw_map,
            },
            "configs": {
                "running_config_checksum": running_checksum,
                "startup_config_checksum": startup_checksum,
                "running_config_path": str(rc_path),
                "startup_config_path": str(sc_path),
            },
            # store selected raw outputs inline for quick access (optionally)
            "raw_inline": {
                "show_version": version_text[:4000],  # keep first N chars inline to avoid huge JSON
            },
            # place-holder for parsed structures; expand as needed
            "parsed": {
                "interfaces": None,
                "vlans": None,
                "mac_table": None,
                "arp_table": None,
                "neighbors": None,
                "routing": None,
                "health": None,
            },
        }

        # Save the JSON snapshot to disk
        snapshot_path = save_snapshot(base_path, device_id, snapshot)
        logger.info(f"Saved snapshot for {device_id} -> {snapshot_path}")
        return snapshot

    finally:
        if conn:
            try:
                conn.disconnect()
            except Exception:
                pass


def diff_last_two_snapshots(device_id: str, base_path: str) -> Dict[str, str]:
    """
    Find the two most recent snapshot JSON files for device_id and return diffs for running-configs.
    Returns dict with keys: 'old_snapshot', 'new_snapshot', 'running_config_diff'
    """
    snaps_dir = Path(base_path) / device_id / "snapshots"
    if not snaps_dir.exists():
        raise FileNotFoundError(f"No snapshots directory for {device_id}")

    snapshot_files = sorted(snaps_dir.glob("*.json"))
    if len(snapshot_files) < 2:
        raise ValueError(f"Need at least two snapshots to compute diff for {device_id}")

    old_path = snapshot_files[-2]
    new_path = snapshot_files[-1]
    old = load_snapshot(str(old_path))
    new = load_snapshot(str(new_path))

    old_rc_path = old["configs"]["running_config_path"]
    new_rc_path = new["configs"]["running_config_path"]

    with open(old_rc_path, "r", encoding="utf-8") as fh:
        old_rc = fh.read()
    with open(new_rc_path, "r", encoding="utf-8") as fh:
        new_rc = fh.read()

    rc_diff = unified_diff(old_rc, new_rc, fromfile=str(old_rc_path), tofile=str(new_rc_path))
    return {"old_snapshot": str(old_path), "new_snapshot": str(new_path), "running_config_diff": rc_diff}


# -------------------------
# Example usage (not executed on import)
# -------------------------
if __name__ == "__main__":
    """
    Example usage notes (do NOT hardcode credentials in code; use env or vault):

    from inventory_collector import make_device_params, collect_device_snapshot, diff_last_two_snapshots

    params = make_device_params(host="192.0.2.1", username="neteng", password="...", secret="enablepwd")
    snapshot = collect_device_snapshot(params, save_base="/var/backups/network")
    print("Snapshot saved:", snapshot["metadata"]["raw_outputs"])

    diff = diff_last_two_snapshots(device_id="192.0.2.1", base_path="/var/backups/network")
    print(diff["running_config_diff"])
    """
    pass