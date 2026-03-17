# terraform/outputs.tf

output "server_ips" {
  value = {
    for i, node in fabric_node.server :
    node.name => node.management_ip
  }
}

output "ssh_key_path" {
  value = fabric_slice.lab.private_key_file
}