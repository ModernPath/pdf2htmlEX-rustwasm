terraform {
  required_version = ">= 1.5.0"

  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }
  }

  backend "s3" {
    bucket         = "ode-terraform-state"
    key            = "infrastructure/terraform.tfstate"
    region         = "us-east-1"
    encrypt        = true
    dynamodb_table = "ode-terraform-locks"
  }
}

provider "aws" {
  region = var.aws_region

  default_tags {
    tags = {
      Project     = "ODE"
      Environment = var.environment
      ManagedBy   = "Terraform"
    }
  }
}

locals {
  name_prefix = "${var.project}-${var.environment}"
}

data "aws_availability_zones" "available" {
  state = "available"
}

module "vpc" {
  source = "./modules/vpc"

  name_prefix = local.name_prefix
  az_count    = 3
  azs         = slice(data.aws_availability_zones.available.names, 0, 3)

  cidr_block = "10.0.0.0/16"

  public_subnet_cidr_blocks   = ["10.0.0.0/20", "10.0.16.0/20", "10.0.32.0/20"]
  private_subnet_cidr_blocks  = ["10.0.64.0/20", "10.0.80.0/20", "10.0.96.0/20"]
  isolated_subnet_cidr_blocks = ["10.0.128.0/20", "10.0.144.0/20", "10.0.160.0/20"]

  enable_nat_gateway = true
  single_nat_gateway = false
}

module "eks" {
  source = "./modules/eks"

  name_prefix      = local.name_prefix
  vpc_id           = module.vpc.vpc_id
  vpc_cidr         = module.vpc.vpc_cidr
  public_subnet_ids = module.vpc.public_subnet_ids
  private_subnet_ids = module.vpc.private_subnet_ids
  isolated_subnet_ids = module.vpc.isolated_subnet_ids

  eks_version = "1.29"

  system_node_group_config = {
    instance_types = ["t3.medium"]
    min_size       = 2
    max_size       = 3
    desired_size   = 2
  }

  worker_node_group_config = {
    instance_types = ["t3.large", "t3a.large"]
    min_size       = 2
    max_size       = 10
    desired_size   = 3
  }
}

module "rds" {
  source = "./modules/rds"

  name_prefix        = local.name_prefix
  subnet_ids         = module.vpc.isolated_subnet_ids
  vpc_id             = module.vpc.vpc_id
  security_group_ids = [module.eks.cluster_security_group_id]

  engine_version = "15.4"
  instance_class = "db.t3.medium"
  allocated_storage = 20
  max_allocated_storage = 100
  
  database_name = var.database_name
  master_username = var.database_username
  master_password = var.database_password

  multi_az = true
}

module "redis" {
  source = "./modules/redis"

  name_prefix        = local.name_prefix
  subnet_ids         = module.vpc.isolated_subnet_ids
  vpc_id             = module.vpc.vpc_id
  security_group_ids = [module.eks.cluster_security_group_id]

  node_type = "cache.t3.medium"
  num_cache_nodes = 2
  replication_group_id = "${local.name_prefix}-redis"
  
  engine_version = "7.0"
}

module "s3" {
  source = "./modules/s3"

  name_prefix = local.name_prefix
  
  bucket_names = [
    "${local.name_prefix}-documents",
    "${local.name_prefix}-exports",
    "${local.name_prefix}-fonts",
  ]
}

resource "aws_ecr_repository" "images" {
  for_each = toset(["ode-api", "ode-worker", "ode-core"])

  name                 = "${local.name_prefix}-${each.value}"
  image_tag_mutability = "MUTABLE"

  force_delete = false

  image_scanning_configuration {
    scan_on_push = true
  }

  encryption_configuration {
    encryption_type = "AES256"
  }
}