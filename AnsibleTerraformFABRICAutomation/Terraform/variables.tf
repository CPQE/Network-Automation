# terraform/variables.tf

variable "project_id" {
  type = string
}

variable "slice_name" {
  type    = string
  default = "lab"
}

variable "site" {
  type    = string
  default = "EDUKY"
}

variable "node_cores" {
  type    = number
  default = 2
}

variable "node_ram" {
  type    = number
  default = 8
}

variable "node_disk" {
  type    = number
  default = 10
}

variable "node_image" {
  type    = string
  default = "default_ubuntu_20"
}

variable "lab_number" {
  type    = number
  default = 5
}