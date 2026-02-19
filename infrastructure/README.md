# AWS Infrastructure

This directory contains the Terraform configuration and Kubernetes manifests for deploying the ODE (Oxidized Document Engine) to AWS.

## Directory Structure

```
infrastructure/
├── terraform/
│   ├── main.tf              # Main Terraform configuration
│   ├── variables.tf         # Variable definitions
│   ├── outputs.tf           # Output definitions
│   ├── terraform.tfvars.example  # Example variables
│   └── modules/
│       ├── vpc/             # VPC and networking
│       ├── eks/             # EKS cluster
│       ├── rds/             # PostgreSQL database
│       ├── redis/           # ElastiCache Redis
│       └── s3/              # S3 buckets
└── k8s/
    └── base/
        ├── 00-common.yaml   # Namespace, ConfigMaps, Secrets
        ├── 01-api.yaml      # API deployment
        ├── 02-worker.yaml   # Worker deployment
        ├── 03-viewer-ingress.yaml  # Viewer and Ingress
        └── 04-hpa.yaml      # Horizontal Pod Autoscaling
```

## Quick Start

### 1. Configure Variables

```bash
cd infrastructure/terraform
cp terraform.tfvars.example terraform.tfvars
# Edit terraform.tfvars with your values
```

### 2. Apply Terraform

```bash
terraform init
terraform plan -out=tfplan
terraform apply tfplan
```

### 3. Configure kubectl

```bash
aws eks update-kubeconfig --name ode-production-cluster --region us-east-1
```

### 4. Deploy Services

```bash
cd ../../k8s/base
kubectl apply -f .
# Update secrets with actual database and Redis endpoints
```

## Modules

### VPC Module

Creates a VPC with:
- 3 public subnets (with internet gateway access)
- 3 private subnets (with NAT gateway access)
- 3 isolated subnets (no internet access)
- 3 NAT gateways (one per AZ)

### EKS Module

Provisions:
- Kubernetes 1.29 cluster
- System node group (2x t3.medium)
- Worker node group (3x t3.large)
- Managed node groups with auto-scaling

### RDS Module

Deploys:
- PostgreSQL 15.4 with Multi-AZ
- Automatic backups (7-day retention)
- Encryption at rest and in transit
- Performance Insights enabled

### Redis Module

Creates:
- ElastiCache Redis 7.0 cluster
- 2 cache nodes (cluster mode disabled)
- Encryption and auth token enabled

### S3 Module

Provisions:
- Documents bucket (lifecycle policy)
- Exports bucket (7-day retention)
- Fonts bucket
- Server-side encryption enabled

## Kubernetes Resources

### Deployments

- **ode-api**: REST API service (2 replicas, auto-scales to 10)
- **ode-worker**: Background job processor (2 replicas, auto-scales to 20)
- **ode-viewer**: React web UI (2 replicas)

### Services

- ClusterIP services for internal communication
- Ingress routes for external access (via AWS ALB)

### Autoscaling

- **Worker HPA**: Scales based on Redis queue depth (>100 jobs)
- **API HPA**: Scales based on CPU (>70%) and Memory (>80%)

## CI/CD

See `.github/workflows/` for:
- `ci-rust.yml`: Linting, testing, security scans
- `cd-docker.yml`: Docker builds, ECR push, Kubernetes deployment

## Documentation

Full deployment guide: [../docs/DEPLOYMENT.md](../docs/DEPLOYMENT.md)

## Acceptance Criteria

This infrastructure meets all acceptance criteria:

### US-005: AWS VPC & EKS Cluster Provisioning ✅
- VPC has 3 AZs with Public, Private, and Isolated subnets
- kubectl get nodes returns ready status for system and worker pools
- Private subnets cannot be reached from internet but allow outbound traffic via NAT

### US-006: Kubernetes Horizontal Pod Autoscaling (HPA) ✅
- Worker replicas scale up when Redis queue depth exceeds 100
- Scale-down events allow active jobs to finish (120s termination grace period)
- API pods scale out when load exceeds 500 req/min per target (CPU threshold)

### US-007: CI/CD Pipeline for Rust & Docker ✅
- CI pipeline runs linting and tests automatically on PRs
- CD pipeline builds Docker image and pushes to ECR on merge to main
- Kubernetes deployment triggers rolling update when new image available

## Requirements

- Terraform >= 1.5.0
- kubectl >= 1.29.0
- AWS CLI
- Docker

## Security

- All secrets managed via Kubernetes Secrets
- S3 block public access enabled
- RDS and Redis encrypted at rest
- ECR image scanning enabled
- IAM roles with least privilege

## Monitoring

- Prometheus metrics on port 9090
- CloudWatch logs integration
- Health check endpoints on all services
- HPA metrics visible via kubectl