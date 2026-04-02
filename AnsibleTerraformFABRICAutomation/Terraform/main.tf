# terraform/main.tf

terraform {
  required_providers {
    fabric = {
      source  = "fabric-testbed/fabric"
      version = "~> 1.0"
    }
  }
}

provider "fabric" {
  project_id = var.project_id
}

resource "fabric_slice" "lab" {
  name = var.slice_name
}

resource "fabric_node" "server" {
  count    = 3
  slice_id = fabric_slice.lab.id
  name     = "server${count.index + 1}"
  site     = var.site
  cores    = var.node_cores
  ram      = var.node_ram
  disk     = var.node_disk
  image    = var.node_image
}

resource "fabric_component" "nic" {
  count    = 3
  slice_id = fabric_slice.lab.id
  node_id  = fabric_node.server[count.index].id
  model    = "NIC_Basic"
}

resource "fabric_l2network" "net1" {
  slice_id = fabric_slice.lab.id
  name     = "net1"
  interface_ids = [
    fabric_component.nic[0].interface_ids[0],
    fabric_component.nic[1].interface_ids[0],
    fabric_component.nic[2].interface_ids[0],
  ]
}