# Performance Test Plan: Oxidized Document Engine (ODE)

## 1. Project Overview
This test plan defines the strategy, process, and benchmarks for validating the performance of the **Oxidized Document Engine (ODE)**. The focus is on ensuring the Rust-based core and WebAssembly (Wasm) execution paths meet high-fidelity rendering requirements under significant load while maintaining memory safety and system stability.

---

## 2. Performance Objectives & Acceptance Criteria
- [ ] **Throughput**: System must handle 50 concurrent PDF-to-HTML conversions per pod without exceeding 80% CPU utilization.
- [ ] **Latency**: 95th percentile (P95) for a standard 10-page PDF (approx. 2MB) must be under 2.5 seconds.
- [ ] **Memory Stability**: No memory growth (leaks) observed during a 12-hour soak test.
- [ ] **Scalability**: EKS Horizontal Pod Autoscaler (HPA) must trigger and successfully spin up new pods within 45 seconds of sustained load (>70% CPU).
- [ ] **Wasm Execution**: Client-side Wasm conversion must complete within 1.5x of the server-side execution time for the same document.

---

## 3. Test Environment & Tools
- **Infrastructure**: AWS EKS (Staging Environment), m5.xlarge nodes.
- **Load Generation**: `k6` (distributed) using TypeScript scripts.
- **Monitoring**: Prometheus (metrics collection), Grafana (dashboards), OpenTelemetry (tracing).
- **Profiling**: `cargo-flamegraph` for Rust bottlenecks, Chrome DevTools for Wasm profiling.

---

## 4. Test Cases

### TC-001: Baseline Single-User Latency
- **Description**: Measure the response time for a single conversion request to establish a "clean" benchmark.
- **Preconditions**: System is idle; no other jobs in Redis queue.
- **Test Steps**:
    1. Upload a standard 5MB PDF via the Axum API `/v1/convert` endpoint.
    2. Measure time from `POST` request to `200 OK` response with HTML payload.
    3. Repeat 10 times to calculate average.
- **Expected Result**: Average latency < 1.5s for 5MB file.
- **Actual Files**: `src/api/handlers.rs`, `src/engine/core.rs`

### TC-002: Sustained Load Throughput
- **Description**: Determine if the system maintains P95 targets under expected peak load.
- **Preconditions**: Load generator configured for 50 Virtual Users (VUs).
- **Test Steps**:
    1. Ramp up to 50 VUs over 2 minutes.
    2. Maintain 50 VUs for 10 minutes, continuously submitting 2MB PDFs.
    3. Monitor Prometheus for `http_request_duration_seconds`.
- **Expected Result**: P95 latency remains < 3.0s; Error rate < 0.1%.
- **Actual Files**: `src/api/router.rs`, `src/queue/redis_worker.rs`

### TC-003: Stress Test (Breaking Point)
- **Description**: Identify the maximum concurrent requests a single ODE pod can handle before failure.
- **Preconditions**: HPA disabled to isolate a single pod.
- **Test Steps**:
    1. Start with 10 VUs.
    2. Increase by 10 VUs every 30 seconds until the error rate exceeds 5% or latency > 10s.
    3. Record CPU and Memory at the breaking point.
- **Expected Result**: Pod should handle at least 80 concurrent conversions before degrading.
- **Actual Files**: `src/main.rs` (Tokio runtime config)

### TC-004: Memory Leak / Soak Test
- **Description**: Verify Rust memory safety and prevent fragmentation over time.
- **Preconditions**: 12-hour test window.
- **Test Steps**:
    1. Apply a constant load of 10 conversions/minute.
    2. Monitor `process_resident_memory_bytes` via Prometheus.
    3. Analyze memory delta between hour 1 and hour 12.
- **Expected Result**: Memory usage should plateau and remain stable (+/- 5% variance).
- **Actual Files**: `src/engine/wasm_binding.rs` (Wasm memory management)

### TC-005: Large Document Handling (Edge Case)
- **Description**: Test system behavior with a 500MB, 1000+ page PDF.
- **Preconditions**: Increase API request body limit in Axum.
- **Test Steps**:
    1. Submit a 500MB PDF to the conversion endpoint.
    2. Monitor Redis task visibility timeout and worker heartbeat.
- **Expected Result**: System processes the file without OOM (Out of Memory) kills; worker remains responsive.
- **Actual Files**: `src/api/middleware.rs` (Payload limits), `src/engine/parser.rs`

### TC-006: Wasm Client-Side Performance Benchmark
- **Description**: Measure the efficiency of the Wasm-compiled core in the browser.
- **Preconditions**: React frontend loaded in Playwright with performance tracing enabled.
- **Test Steps**:
    1. Trigger Wasm conversion of a 2MB PDF in the browser.
    2. Measure time from file input to DOM rendering.
    3. Compare against server-side benchmarks.
- **Expected Result**: Conversion completes in < 4s on standard desktop hardware.
- **Actual Files**: `src/wasm/lib.rs`, `ui/components/WasmConverter.tsx`

### TC-007: Redis Task Queue Latency
- **Description**: Measure the overhead of the asynchronous task distribution.
- **Preconditions**: Redis is populated with 5,000 pending jobs.
- **Test Steps**:
    1. Measure time from `Job Created` (Postgres) to `Job Picked Up` (Worker).
- **Expected Result**: Queue pickup latency < 100ms.
- **Actual Files**: `src/queue/producer.rs`, `src/queue/consumer.rs`

### TC-008: Auto-scaling (HPA) Validation
- **Description**: Ensure the infrastructure responds to load by scaling.
- **Preconditions**: EKS cluster with HPA configured (Target: 70% CPU).
- **Test Steps**:
    1. Flood the API with 200 concurrent VUs.
    2. Watch `kubectl get hpa -w`.
    3. Record time for new pods to reach `Running` state.
- **Expected Result**: Cluster scales from 2 to 10 pods; load redistributes successfully.
- **Actual Files**: `terraform/eks/hpa.yaml`, `docker/Dockerfile`

### TC-009: Database Metadata Contention
- **Description**: Test PostgreSQL performance under high write volume of job metadata.
- **Preconditions**: 100 concurrent workers writing job status updates.
- **Test Steps**:
    1. Simulate high-frequency status updates (Started, Processing Page X, Completed).
    2. Monitor DB lock contention and CPU.
- **Expected Result**: DB CPU < 40%; no deadlocks observed.
- **Actual Files**: `src/db/models.rs`, `src/db/migrations/`

### TC-010: Graceful Degradation (Circuit Breaking)
- **Description**: Verify system behavior when Redis or Postgres is unreachable.
- **Preconditions**: Sentry integration active.
- **Test Steps**:
    1. Simulate a Redis connection failure during high load.
    2. Attempt to submit a new job.
- **Expected Result**: API returns `503 Service Unavailable` quickly; Sentry captures the event; no pod crashes.
- **Actual Files**: `src/api/errors.rs`, `src/queue/connection.rs`

---

## 5. Test Data
- **Standard Set**: 100 PDFs ranging from 1MB to 10MB (Mixed text and vector graphics).
- **Complex Set**: 10 PDFs with heavy SVG usage, embedded fonts, and complex transparency layers.
- **Malicious/Edge Set**: Corrupted PDF headers, 0-byte files, and "Zip Bomb" style PDFs.

---

## 6. Verification Checklist
- [ ] **TC-001 to TC-010** executed in the Staging environment.
- [ ] **Flamegraphs** generated for any Rust function taking > 20% of execution time.
- [ ] **Grafana Dashboard** exported showing P95, Throughput, and Error Rate for the duration of the tests.
- [ ] **Sentry** logs reviewed for any unhandled exceptions during stress testing.
- [ ] **Terraform** configs updated if resource limits (CPU/Mem) need adjustment based on findings.

---

## 7. Dependencies & Verification Steps
### Dependencies
1.  **Completion of ODE Core**: Rust engine must be feature-complete for PDF parsing.
2.  **Infrastructure Up**: EKS and Redis/Postgres must be provisioned via Terraform.
3.  **Observability**: Prometheus/Grafana sidecars must be functional.

### Verification Steps (How to run)
1.  **Deploy**: `kubectl apply -f k8s/staging/`
2.  **Run k6**: `k6 run ./performance/scripts/load_test.js --env BASE_URL=https://api.staging.ode.com`
3.  **Analyze**: Open Grafana Performance Dashboard and verify metrics against Section 2 (Acceptance Criteria).
4.  **Profile**: `kubectl exec <pod_name> -- cargo flamegraph --pid 1` during load.