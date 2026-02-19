output "bucket_names" {
  description = "List of bucket names"
  value       = [for bucket in aws_s3_bucket.this : bucket.id]
}

output "bucket_arns" {
  description = "List of bucket ARNs"
  value       = [for bucket in aws_s3_bucket.this : bucket.arn]
}