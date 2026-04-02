# ansible/generate_inventory.py

import argparse
import json
import subprocess
import sys
from pathlib import Path

def get_terraform_outputs(tf_dir):
    result = subprocess.run(
        ["terraform", "output", "-json"],
        cwd=tf_dir,
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        print(f"ERROR: terraform output failed:\n{result.stderr}", file=sys.stderr)
        sys.exit(1)
    return json.loads(result.stdout)


def write_inventory(outputs, out_path):
    server_ips = outputs["server_ips"]["value"]
    ssh_key    = outputs["ssh_key_path"]["value"]

    lines = ["[servers]"]
    for name, ip in sorted(server_ips.items()):
        lines.append(
            f"{name} ansible_host={ip} ansible_user=ubuntu "
            f"ansible_ssh_private_key_file={ssh_key} "
            f"ansible_ssh_common_args='-o StrictHostKeyChecking=no'"
        )
    lines += ["", "[servers:vars]", "ansible_python_interpreter=/usr/bin/python3"]

    Path(out_path).write_text("\n".join(lines) + "\n")
    print(f"Inventory written to: {out_path}")
    for name, ip in sorted(server_ips.items()):
        print(f"  {name}: {ip}")


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--tf-dir", default="../terraform")
    parser.add_argument("--out",    default="inventory.ini")
    args = parser.parse_args()
    write_inventory(get_terraform_outputs(args.tf_dir), args.out)


if __name__ == "__main__":
    main()