variable "name_prefix" {
  description = "Prefix for resource names"
  type        = string
}

variable "bucket_names" {
  description = "List of bucket names to create"
  type        = list(string)
}