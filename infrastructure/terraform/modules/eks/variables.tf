variable "name_prefix" {
  description = "Prefix for resource names"
  type        = string
}

variable "vpc_id" {
  description = "VPC ID"
  type        = string
}

variable "vpc_cidr" {
  description = "VPC CIDR block"
  type        = string
}

variable "public_subnet_ids" {
  description = "Public subnet IDs"
  type        = list(string)
}

variable "private_subnet_ids" {
  description = "Private subnet IDs"
  type        = list(string)
}

variable "isolated_subnet_ids" {
  description = "Isolated subnet IDs"
  type        = list(string)
}

variable "eks_version" {
  description = "EKS version"
  type        = string
  default     = "1.29"
}

variable "system_node_group_config" {
  description = "System node group configuration"
  type = object({
    instance_types = list(string)
    min_size       = number
    max_size       = number
    desired_size   = number
  })
}

variable "worker_node_group_config" {
  description = "Worker node group configuration"
  type = object({
    instance_types = list(string)
    min_size       = number
    max_size       = number
    desired_size   = number
  })
}