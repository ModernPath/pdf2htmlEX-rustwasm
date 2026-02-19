# Prometheus Monitoring Stack

## Overview

The Prometheus monitoring stack provides metrics collection, aggregation, and custom metrics support for Kubernetes autoscaling.

## Components

### 1. Prometheus Server (`05-prometheus.yaml`)

Scrapes metrics from:
- Kubernetes pods with `prometheus.io/scrape="true"` annotation
- Kubernetes API server
- Redis exporter
- Custom application endpoints on port 9090

**Storage**: 15 days retention in emptyDir (persistent storage recommended for production)

**Access**: ClusterIP service on port 9090

### 2. Prometheus Adapter (`06-prometheus-adapter.yaml`)

Provides custom metrics API for HPA:
- Resource metrics: CPU, Memory
- Custom metrics: Redis metrics (redis_db_keys, redis_connected_clients, etc.)

**API Server**: `custom.metrics.k8s.io/v1beta1`

**Configuration**: Automatically scrapes all `redis_*` metrics from Prometheus

### 3. Redis Exporter (`07-redis-exporter.yaml`)

Exports Redis metrics for monitoring and autoscaling:
- `redis_db_keys` - Total keys in database (used for queue depth)
- `redis_connected_clients` - Active connections
- `redis_memory_used_bytes` - Memory usage
- `redis_commands_processed_total` - Total operations

**Endpoint**: `redis-exporter.ode.svc.cluster.local:9121/metrics`

## HPA Custom Metrics

### Worker Autoscaling

Current metric: `redis_db_keys` (average value across all worker pods)

**Rationale**: The number of keys in Redis databases correlates with the pending job queue depth. This is a reliable proxy for workload size.

**Scaling behavior**:
- Scale up when average keys per pod > 100
- Scale down stabilization: 600s (allows jobs to complete)
- Max replicas: 20

### API Autoscaling

Current metrics: CPU and Memory utilization

**Rationale**: Request rate (~500 req/min) correlates with CPU usage for I/O-bound services. Using CPU is more reliable than implementing custom request counting.

**Scaling behavior**:
- Scale up at 70% CPU or 80% memory
- Scale down stabilization: 300s
- Max replicas: 10

## Deployment Order

```bash
kubectl apply -f k8s/base/00-common.yaml
kubectl apply -f k8s/base/05-prometheus.yaml
kubectl apply -f k8s/base/07-redis-exporter.yaml
kubectl apply -f k8s/base/06-prometheus-adapter.yaml
kubectl apply -f k8s/base/04-hpa.yaml
```

## Verification

### Check Prometheus is scraping:

```bash
kubectl port-forward -n ode svc/prometheus 9090:9090
# Visit http://localhost:9090/targets
```

### Check custom metrics are available:

```bash
kubectl get --raw /apis/custom.metrics.k8s.io/v1beta1/namespaces/ode/pods/redis_db_keys
```

### Check HPA is working:

```bash
kubectl get hpa -n ode
kubectl describe hpa ode-worker -n ode
```

## Application Metrics

Services expose metrics on port 9090 with Prometheus format:

```rust
// Example in ode-worker:
use prometheus::{Counter, Histogram, Registry, register_counter};

lazy_static! {
    static ref QUEUE_DEPTH: Gauge = register_gauge!(
        "worker_queue_depth",
        "Current queue depth"
    ).unwrap();

    static ref PROCESSED_JOBS: Counter = register_counter!(
        "worker_processed_jobs_total",
        "Total jobs processed"
    ).unwrap();
}

// Update metrics
QUEUE_DEPTH.set(42);
PROCESSED_JOBS.inc();
```

## Troubleshooting

### HPA not scaling

1. Check custom metrics API is working:
   ```bash
   kubectl get apiservice v1beta1.custom.metrics.k8s.io
   ```

2. Check Prometheus adapter logs:
   ```bash
   kubectl logs -n ode deployment/prometheus-adapter
   ```

3. Verify Prometheus has data:
   ```bash
   kubectl exec -n ode deployment/prometheus -- promtool query instant 'redis_db_keys'
   ```

### Metrics not appearing

1. Verify annotations on pods:
   ```bash
   kubectl get pods -n ode -o json | jq '.items[].metadata.annotations'
   ```

2. Check Prometheus configuration is valid:
   ```bash
   kubectl exec -n ode deployment/prometheus -- promtool check config /etc/prometheus/prometheus.yml
   ```

3. Verify Redis exporter can connect:
   ```bash
   kubectl exec -n ode deployment/redis-exporter -- curl http://localhost:9121/metrics
   ```

## Production Recommendations

1. **Persistent Storage**: Use PVC for Prometheus data retention
2. **Alerting**: Add AlertManager for notifications
3. **Grafana**: Add visualization dashboard (optional)
4. **Retention**: Increase to 30 days for production
5. **High Availability**: Run Prometheus with Thanos or Cortex for multi-cluster