# FABRIC Lab — Terraform + Ansible
Replaces the  `fablib` Python script setup with a clean two-stage pipeline:
Terraform Provision (creates a slice, its nodes, nics, and an L2 network)
Ansible Configuration (Installs packages, uploads files, and install Rust)
```
fabric-lab/
├── terraform/
│   ├── main.tf                  # slice, nodes, NICs, L2 network
│   ├── variables.tf             # all tuneable parameters
│   ├── outputs.tf               # server IPs + SSH key path
│   └── terraform.tfvars.example # copy → terraform.tfvars and fill in
└── ansible/
    ├── playbook.yml             # apt installs, file uploads, Rust setup
    ├── ansible.cfg              # sane defaults (no host key checking, etc.)
    └── generate_inventory.py    # reads terraform output → writes inventory.ini
```

---

## Prerequisites

```bash
# Terraform FABRIC provider
terraform init   # inside terraform/

# Python 3 (for the glue script — stdlib only, no pip needed)
python3 --version

# Ansible
pip install ansible
```

---

## Step 1 — Provision with Terraform

```bash
cd terraform
cp terraform.tfvars.example terraform.tfvars
# edit terraform.tfvars — set your project_id, lab_number, etc.

terraform init
terraform plan
terraform apply
```

Terraform will create the slice and block until all nodes are ready.

---

## Step 2 — Generate Ansible Inventory

```bash
cd ../ansible
python3 generate_inventory.py
# reads ../terraform outputs → writes inventory.ini
```

Check that `inventory.ini` looks right before continuing.

---

## Step 3 — Configure nodes with Ansible

Make sure `Lab{number}.zip` and `rustup-init.sh` are in the `ansible/` directory, then:

```bash
ansible-playbook playbook.yml
```

To use a different lab number:
```bash
ansible-playbook playbook.yml -e "lab_number=6"
```

---

## Teardown

```bash
cd terraform
terraform destroy
```

This cleanly deletes the slice and all associated resources on FABRIC.

---

## Changing node count

Edit `main.tf` — the `count = 3` on `fabric_node.server` and `fabric_component.nic`
controls how many servers are provisioned. The L2 network `interface_ids` list
will need to be updated to match.