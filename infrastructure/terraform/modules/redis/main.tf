resource "aws_elasticache_replication_group" "this" {
  replication_group_id          = var.replication_group_id
  replication_group_description = "${var.name_prefix} Redis cluster"
  node_type                     = var.node_type
  num_cache_clusters            = var.num_cache_nodes
  engine                        = "redis"
  engine_version                = var.engine_version
  port                          = 6379
  parameter_group_name          = aws_elasticache_parameter_group.this.name
  subnet_group_name             = aws_elasticache_subnet_group.this.name
  security_group_ids            = var.security_group_ids

  at_rest_encryption_enabled  = true
  transit_encryption_enabled  = true
  auth_token                  = var.auth_token

  multi_az_enabled = true

  automatic_failover_enabled = true
  snapshot_retention_limit   = 7
  snapshot_window           = "03:00-05:00"

  maintenance_window = "sun:05:00-sun:06:00"

  cluster_mode {
    replicas_per_node_group = 1
    num_node_groups         = 1
  }

  tags = {
    Name = "${var.name_prefix}-redis"
  }
}

resource "aws_elasticache_parameter_group" "this" {
  name        = "${lower(var.name_prefix)}-redis-params"
  family      = "redis7"
  description = "${var.name_prefix} Redis parameter group"

  parameter {
    name  = "maxmemory-policy"
    value = "allkeys-lru"
  }

  parameter {
    name  = "notify-keyspace-events"
    value = "Ex"
  }
}

resource "aws_elasticache_subnet_group" "this" {
  name        = "${var.name_prefix}-redis-subnet-group"
  description = "${var.name_prefix} Redis subnet group"
  subnet_ids  = var.subnet_ids
}

resource "aws_security_group" "this" {
  name_prefix = "${var.name_prefix}-redis-sg-"
  description = "Redis security group"
  vpc_id      = var.vpc_id

  tags = {
    Name = "${var.name_prefix}-redis-sg"
  }
}

resource "aws_security_group_rule" "ingress" {
  description              = "Allow Redis access from EKS"
  from_port                = 6379
  to_port                  = 6379
  protocol                 = "tcp"
  security_group_id        = aws_security_group.this.id
  source_security_group_id = var.security_group_ids[0]
  type                     = "ingress"
}