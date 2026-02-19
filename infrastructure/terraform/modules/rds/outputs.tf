output "endpoint" {
  description = "RDS endpoint"
  value       = aws_db_instance.this.endpoint
  sensitive   = true
}

output "database_name" {
  description = "Database name"
  value       = aws_db_instance.this.db_name
}

output "port" {
  description = "Database port"
  value       = aws_db_instance.this.port
}

output "username" {
  description = "Master username"
  value       = aws_db_instance.this.username
  sensitive   = true
}

output "instance_id" {
  description = "Database instance ID"
  value       = aws_db_instance.this.id
}

output "security_group_id" {
  description = "Security group ID"
  value       = aws_security_group.this.id
}