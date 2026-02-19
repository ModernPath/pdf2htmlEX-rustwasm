# AWS Infrastructure Deployment Guide

This guide documents the complete AWS hybrid cloud infrastructure for the ODE (Oxidized Document Engine) project.

## Overview

The infrastructure consists of:
- **VPC** with 3 availability zones (Public, Private, and Isolated subnets)
- **EKS** Kubernetes 1.29 cluster with system and worker node pools
- **RDS** PostgreSQL 15.4 with Multi-AZ deployment
- **ElastiCache** Redis 7.0 cluster
- **S3** buckets for document storage
- **ECR** repositories for container images

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                        AWS Region                             │
└─────────────────────────────────────────────────────────────┘
                              │
        ┌─────────────────────┴─────────────────────┐
        │                      VPC (10.0.0.0/16)    │
        └─────────────────────┬─────────────────────┘
                              │
        ┌─────────────────────┴─────────────────────────────────┐
        │                                                      │
    ┌───▼────┐  ┌──────────────────┐  ┌─────────────────────┐  │
    │Public  │  │     Private     │  │      Isolated        │  │
    │Subnet  │  │     Subnet      │  │      Subnet          │  │
    │(AZ 1-3)│  │    (AZ 1-3)     │  │     (AZ 1-3)         │  │
    └───┬────┘  └────────┬─────────┘  └──────────┬──────────┘  │
        │               │                       │               │
        │          ┌────▼──────┐          ┌─────▼─────┐       │
        │          │   EKS     │          │    RDS    │       │
        │          │  Cluster  │          │PostgreSQL│       │
        │          │           │          └───────────┘       │
        │          │  ○ System │                              │
        │          │  ○ Worker │          ┌─────▼─────┐       │
        │          └───────────┘          │   Redis   │       │
        │                                 └───────────┘       │
        │                                                      │
        ├──────────────────────────────────────────────────────┤
        │                  Internet Gateway                   │
        ▼
               ┌──────────────────────┐
               │      NAT Gateway     │
               └──────────────────────┘
```

## Prerequisites

1. **AWS Account** with appropriate permissions
2. **Terraform** >= 1.5.0
3. **kubectl** >= 1.29.0
4. **AWS CLI** configured with credentials
5. **Docker** for local container builds

## Initial Setup

### 1. Configure Terraform Remote State

First, create the S3 bucket and DynamoDB table for Terraform state:

```bash
aws s3api create-bucket \
  --bucket ode-terraform-state \
  --region us-east-1

aws dynamodb create-table \
  --table-name ode-terraform-locks \
  --attribute-definitions AttributeName=LockID,AttributeType=S \
  --key-schema AttributeName=LockID,KeyType=HASH \
  --billing-mode PAY_PER_REQUEST \
  --region us-east-1 \
  --tag-keys ManagedBy --tag-values Terraform
```

### 2. Configure Variables

Copy the example variables file:

```bash
cd infrastructure/terraform
cp terraform.tfvars.example terraform.tfvars
```

Edit `terraform.tfvars` with your values:

```hcl
aws_region  = "us-east-1"
project     = "ode"
environment = "production"

database_name     = "odedb"
database_username = "odeadmin"
database_password = "CHANGE_ME_SECURE_PASSWORD"
```

⚠️ **Security Note**: Never commit `terraform.tfvars` to version control. It's included in `.gitignore`.

## Deployment Steps

### 1. Initialize Terraform

```bash
cd infrastructure/terraform
terraform init
```

### 2. Plan Infrastructure

```bash
terraform plan -out=tfplan
```

Review the plan carefully before proceeding.

### 3. Apply Infrastructure

```bash
terraform apply tfplan
```

This will provision:
- VPC with 9 subnets (3 public, 3 private, 3 isolated)
- 3 NAT Gateways (one per AZ)
- EKS 1.29 cluster
- EKS node groups (system + worker)
- RDS PostgreSQL Multi-AZ
- ElastiCache Redis cluster
- S3 buckets (documents, exports, fonts)
- ECR repositories

The deployment typically takes 15-20 minutes.

### 4. Configure kubectl

After EKS is ready, configure kubectl:

```bash
aws eks update-kubeconfig --name ode-production-cluster --region us-east-1
kubectl get nodes
```

You should see both system and worker nodes in `Ready` status.

### 5. Deploy Kubernetes Resources

```bash
cd ../../k8s/base

# Update secrets with real values
kubectl apply -f 00-common.yaml
kubectl patch secret ode-secrets -n ode -p '{"stringData":{"database-url":"<RDS_ENDPOINT>","redis-url":"<REDIS_ENDPOINT>"}}'

# Deploy services
kubectl apply -f 01-api.yaml
kubectl apply -f 02-worker.yaml
kubectl apply -f 03-viewer-ingress.yaml
kubectl apply -f 04-hpa.yaml
```

### 6. Verify Deployment

```bash
# Check pods
kubectl get pods -n ode

# Check services
kubectl get svc -n ode

# Check HPA
kubectl get hpa -n ode

# Check ingress
kubectl get ingress -n ode
```

## Verification

### Verify VPC Configuration

```bash
# Check subnets
aws ec2 describe-subnets --filters "Name=vpc-id,Values=$(terraform output -raw vpc_id)"

# Verify private subnets have outbound-only access
# They should route to NAT Gateways but not have public IPs
```

### Verify EKS Cluster

```bash
# List nodes
kubectl get nodes -o wide
# Expected: 2 system nodes + 3 worker nodes in Ready state

# Check cluster version
kubectl version
# Expected: Server Version: v1.29.x
```

### Verify RDS PostgreSQL

```bash
# Get RDS endpoint
terraform output rds_endpoint

# Test connectivity
psql -h <RDS_ENDPOINT> -U odeadmin -d odedb
```

### Verify ElastiCache Redis

```bash
# Get Redis endpoint
terraform output redis_endpoint

# Test connectivity
redis-cli -h <REDIS_ENDPOINT> -a <AUTH_TOKEN> ping
# Expected: PONG
```

### Verify HPA Scaling

```bash
# Check HPA status
kubectl get hpa -n ode

# Simulate load (for testing)
# Worker HPA scales when Redis queue depth > 100
# API HPA scales when CPU > 70% or Memory > 80%
```

## CI/CD Pipeline

### GitHub Secrets Configuration

Configure these secrets in your GitHub repository:

- `AWS_ACCESS_KEY_ID`: AWS access key with programmatic access
- `AWS_SECRET_ACCESS_KEY`: AWS secret key
- `AWS_REGION`: AWS region (default: us-east-1)

### Pipeline Behavior

**CI Pipeline** (`.github/workflows/ci-rust.yml`):
- Runs on PRs and pushes to `main`
- Executes linting (fmt, clippy)
- Runs tests (debug + release)
- Generates code coverage reports
- Performs security audit

**CD Pipeline** (`.github/workflows/cd-docker.yml`):
- Runs on pushes to `main` and tags
- Builds Docker images for all services
- Pushes to ECR
- Scans images for vulnerabilities
- Performs rolling update to EKS
- Runs smoke tests
- Automatic rollback on failure

## Scaling Behavior

### Worker Pod Autoscaling

- **Min replicas**: 2
- **Max replicas**: 20
- **Scale-up trigger**: Redis queue depth > 100 pending jobs
- **Scale-up behavior**: Up to 100% increase every 30s (max 4 pods)
- **Scale-down behavior**: Down to 50% every 60s after 10 min stabilization

 Graceful shutdown: Pods receive SIGTERM, have 120s to complete active jobs

### API Pod Autoscaling

- **Min replicas**: 2
- **Max replicas**: 10
- **Scale-up triggers**: CPU utilization > 70% or Memory > 80%
- **Scale-up behavior**: Up to 100% increase every 60s
- **Scale-down behavior**: Down to 50% every 60s after 5 min stabilization

## Security Considerations

### Network Security

- Private subnets cannot be accessed directly from internet
- Isolated subnets have no internet access (for databases)
- All traffic between services stays within VPC
- NAT Gateways provide outbound-only access for private subnets

### Data Security

- RDS and Redis encrypted at rest
- S3 buckets enforce server-side encryption
- All ECR repositories enforce image scanning
- Secrets stored as Kubernetes Secrets (never in configs)

### IAM Roles

- EKS cluster has dedicated IAM role
- Node groups have worker IAM role with least privileges
- EKS service accounts can use IRSA for fine-grained permissions

## Monitoring & Observability

### Prometheus Metrics

All pods expose metrics on port 9090:
- `ode-api`: Request rate, latency, error rate
- `ode-worker`: Queue depth, processing time, active jobs

### CloudWatch Logs

EKS cluster logs are automatically exported to CloudWatch:
- API server logs
- Controller manager logs
- Scheduler logs

### Health Checks

```bash
# API health endpoint
curl http://api.ode.example.com/health

# Worker health endpoint
curl http://ode-worker.ode.svc.cluster.local:9090/health
```

## Troubleshooting

### Pod Not Starting

```bash
# Check pod status
kubectl describe pod <pod-name> -n ode

# Check logs
kubectl logs <pod-name> -n ode

# Check events
kubectl get events -n ode
```

### HPA Not Scaling

```bash
# Check HPA metrics
kubectl describe hpa <hpa-name> -n ode

# Check Metrics Server
kubectl get apiservice v1beta1.metrics.k8s.io

# Check pod metrics
kubectl top pods -n ode
```

### Database Connection Issues

```bash
# Check security groups
aws ec2 describe-security-groups --group-ids <SG_ID>

# Check RDS endpoint connectivity
nc -zv <RDS_ENDPOINT> 5432

# Check secrets
kubectl get secret ode-secrets -n ode -o yaml
```

## Cost Optimization

### Estimated Monthly Costs

| Service | Configuration | Monthly Cost |
|---------|--------------|--------------|
| VPC | 3 AZs, 9 subnets | ~$5 |
| EKS | $0.10/hour cluster | ~$73 |
| EC2 Nodes | 2x t3.medium + 3x t3.large | ~$150 |
| RDS | db.t3.medium Multi-AZ | ~$100 |
| ElastiCache | cache.t3.medium 2 nodes | ~$60 |
| S3 | 1TB storage + requests | ~$30 |
| ECR | Image storage | ~$10 |
| NAT Gateway | 3x NAT Gateways | ~$90 |
| **Total** | | **~$518/month** |

### Cost Savings Tips

1. Use Spot instances for worker nodes (saves ~70%)
2. Enable S3 lifecycle policies for older documents
3. Scale down worker pods during off-peak hours
4. Use reserved instances for predictable workloads

## Disaster Recovery

### Backup Strategy

- **RDS**: Automated backups retained for 7 days
- **S3**: Versioning enabled on all buckets
- **EKS**: No state to backup (etcd managed by AWS)

### Restore Procedure

```bash
# Restore RDS from snapshot
aws rds restore-db-instance-from-db-snapshot \
  --db-instance-identifier ode-postgres-restored \
  --db-snapshot-identifier <SNAPSHOT_ID>

# Update Kubernetes secret with new endpoint
kubectl patch secret ode-secrets \
  -p '{"stringData":{"database-url":"<NEW_ENDPOINT>"}}'
```

## Maintenance

### Rolling Updates

Kubernetes supports zero-downtime deployments:

```bash
# Automatic via CI/CD
# Or manual update of image tag:
kubectl set image deployment/ode-api \
  ode-api=<ECR_REPO_URL>:new-tag -n ode

# Monitor rollout
kubectl rollout status deployment/ode-api -n ode
```

### Cluster Upgrades

```bash
# Update EKS control plane
aws eks update-cluster-version \
  --name ode-production-cluster \
  --kubernetes-version 1.30 \
  --region us-east-1

# Update node group
aws eks update-nodegroup-version \
  --cluster-name ode-production-cluster \
  --nodegroup-name worker \
  --kubernetes-version 1.30
```

## Cleanup

To destroy all infrastructure:

```bash
cd infrastructure/terraform
terraform destroy
```

⚠️ **Warning**: This will delete all resources, including data in RDS, S3, and Redis.

## Support

For issues or questions:
1. Check CloudWatch logs
2. Review Terraform state: `terraform show`
3. Contact the DevOps team

---

*Last updated: 2026-02-18*