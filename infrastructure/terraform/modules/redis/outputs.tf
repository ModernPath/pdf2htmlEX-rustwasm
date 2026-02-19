output "endpoint" {
  description = "Redis endpoint"
  value       = aws_elasticache_replication_group.this.primary_endpoint_address
  sensitive   = true
}

output "port" {
  description = "Redis port"
  value       = aws_elasticache_replication_group.this.port
}

output "replication_group_id" {
  description = "Replication group ID"
  value       = aws_elasticache_replication_group.this.replication_group_id
}

output "security_group_id" {
  description = "Security group ID"
  value       = aws_security_group.this.id
}