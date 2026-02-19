This document outlines the **Non-Functional Requirements (NFRs)** for the **Oxidized Document Engine (ODE)**. As a senior product manager, these stories are designed to ensure the system meets the rigorous performance, security, and operational standards required for an enterprise-grade document transformation engine.

---

## User Stories: Performance, Security, Scalability, & Operations

### 1. Performance: Sub-Second Core Transformation
**Story ID**: US-NFR-001  
**As a** Frontend Engineer  
**I want** the Rust-based core to transform a standard 5-page PDF to HTML in under 500ms  
**So that** end-users experience near-instantaneous document rendering without perceived lag.

*   **Acceptance Criteria**:
    *   [ ] Benchmarking shows average processing time < 500ms for a "Standard Document" (5 pages, mixed text/vector, < 2MB).
    *   [ ] Memory usage during transformation does not exceed 256MB for the standard document.
    *   [ ] CPU utilization is optimized via Tokio multi-threading for batch requests.
*   **Test Scenarios**:
    *   **Scenario 1**: Upload a 5-page text-heavy PDF. **Expected**: Transformation complete and HTML returned in < 400ms.
    *   **Scenario 2**: Upload a 1-page PDF with complex SVG paths. **Expected**: Transformation complete in < 600ms.
*   **Affected Components**: `ode-core` (Rust), `axum-api` (Middleware).
*   **File References**: `src/engine/transform.rs`, `benches/performance_bench.rs`.
*   **Verification Steps**: Run `cargo bench` and analyze results against the 500ms baseline.

**Story Estimation**:
*   **Complexity**: High
*   **Dependencies**: Finalization of the Rust PDF parser logic.

---

### 2. Security: Memory-Safe PDF Parsing & Sandboxing
**Story ID**: US-NFR-002  
**As a** Security Architect  
**I want** to ensure all PDF parsing occurs within a memory-safe environment with strict resource limits  
**So that** malicious PDF "bombs" or buffer overflow attempts cannot compromise the host infrastructure.

*   **Acceptance Criteria**:
    *   [ ] Zero usage of `unsafe` blocks in the Rust parsing logic, or explicitly audited `unsafe` wrappers.
    *   [ ] Implementation of a "Time-out" wrapper (30s max) for any single conversion job.
    *   [ ] Implementation of a "Zip Bomb" detector that rejects PDFs with highly suspicious compression ratios (>100:1).
*   **Test Scenarios**:
    *   **Scenario 1**: Submit a malformed PDF designed to trigger a buffer overflow. **Expected**: Rust engine returns a `Result::Err` or `Panic` caught by Axum, returning a 400 Bad Request.
    *   **Scenario 2**: Submit a 10GB-decompression-size PDF. **Expected**: Engine terminates process within 2 seconds and returns a resource limit error.
*   **Affected Components**: `ode-core`, `axum-api` (Error Handling).
*   **File References**: `src/security/sanitizer.rs`, `src/engine/limits.rs`.
*   **Verification Steps**: Execute `cargo test` using `proptest` to fuzz the input parser with random byte streams.

**Story Estimation**:
*   **Complexity**: Medium
*   **Dependencies**: None.

---

### 3. Scalability: Dynamic Horizontal Scaling via Queue Depth
**Story ID**: US-NFR-003  
**As a** DevOps Engineer  
**I want** the EKS cluster to scale ODE worker nodes based on the Redis task queue depth  
**So that** we can handle sudden bursts of thousands of document conversions without manual intervention.

*   **Acceptance Criteria**:
    *   [ ] Kubernetes Horizontal Pod Autoscaler (HPA) is configured to trigger on custom Prometheus metrics (Redis queue length).
    *   [ ] System scales from 2 to 20 replicas when queue depth exceeds 100 pending jobs.
    *   [ ] Scale-down occurs gracefully, allowing active jobs to finish before pod termination (SIGTERM handling).
*   **Test Scenarios**:
    *   **Scenario 1**: Inject 500 dummy jobs into Redis. **Expected**: EKS nodes increase within 120 seconds.
    *   **Scenario 2**: Clear queue. **Expected**: Pod count returns to baseline after the cooldown period.
*   **Affected Components**: Terraform (EKS/HPA config), Redis, GitHub Actions (Deployment).
*   **File References**: `infra/terraform/eks_hpa.tf`, `infra/k8s/worker-deployment.yaml`.
*   **Verification Steps**: Monitor `kubectl get hpa -w` during a load test.

**Story Estimation**:
*   **Complexity**: Medium
*   **Dependencies**: Prometheus/Grafana integration (US-NFR-004).

---

### 4. Operational: Observability & "Golden Signal" Monitoring
**Story ID**: US-NFR-004  
**As a** Site Reliability Engineer (SRE)  
**I want** a centralized dashboard tracking Latency, Traffic, Errors, and Saturation  
**So that** we can identify and resolve production bottlenecks before they impact customers.

*   **Acceptance Criteria**:
    *   [ ] Prometheus metrics exported for: `conversion_duration_seconds`, `active_jobs_count`, `error_rate_per_type`.
    *   [ ] Sentry integration captures all Rust panics and Axum 5xx errors with full stack traces.
    *   [ ] Grafana dashboard created with alerts for Error Rate > 1% and Latency P99 > 2s.
*   **Test Scenarios**:
    *   **Scenario 1**: Trigger a forced 500 error in the API. **Expected**: Alert appears in Sentry and Grafana error panel increments.
    *   **Scenario 2**: Run a load test. **Expected**: P99 latency is visible and accurate in Grafana.
*   **Affected Components**: Axum (Instrumentation), Prometheus, Sentry.
*   **File References**: `src/monitoring/telemetry.rs`, `infra/grafana/dashboards/ode_main.json`.
*   **Verification Steps**: Access the Grafana production URL and verify data flow from the `dev` environment.

**Story Estimation**:
*   **Complexity**: Low
*   **Dependencies**: OpenTelemetry setup in Axum.

---

### 5. Performance: Client-Side Wasm Offloading
**Story ID**: US-NFR-005  
**As a** Backend Developer  
**I want** to provide a WebAssembly (Wasm) build of the engine for the React component  
**So that** we can offload computation to the user's browser for small documents, reducing server costs.

*   **Acceptance Criteria**:
    *   [ ] `wasm-pack` build pipeline produces a valid `.wasm` and `.js` glue-code package.
    *   [ ] React component detects client capabilities and executes conversion locally for files < 1MB.
    *   [ ] Visual fidelity of Wasm-rendered HTML matches server-side rendered HTML 1:1.
*   **Test Scenarios**:
    *   **Scenario 1**: User uploads a 500KB PDF. **Expected**: Network tab shows no POST to `/convert`; conversion happens in worker thread.
    *   **Scenario 2**: Run Playwright visual regression test on Wasm vs. Server output. **Expected**: 0% pixel difference.
*   **Affected Components**: `ode-wasm`, `react-viewer-library`.
*   **File References**: `src/lib.rs` (with `[lib] crate-type = ["cdylib"]`), `packages/react-component/src/WasmEngine.ts`.
*   **Verification Steps**: Check browser console for "Wasm Engine Initialized" and verify no API calls for small files.

**Story Estimation**:
*   **Complexity**: High
*   **Dependencies**: Rust core must be compatible with `wasm32-unknown-unknown` (no direct OS syscalls).

---

### 6. Security: API Authentication & Rate Limiting
**Story ID**: US-NFR-006  
**As an** Enterprise Content Manager  
**I want** all API requests to be authenticated via JWT and rate-limited per API key  
**So that** our document processing credits are not exhausted by unauthorized third parties.

*   **Acceptance Criteria**:
    *   [ ] Axum middleware validates JWTs against the PostgreSQL user store.
    *   [ ] Redis-backed rate limiter restricts users to 100 requests/minute (configurable).
    *   [ ] Failed auth returns 401 Unauthorized; exceeded limits return 429 Too Many Requests.
*   **Test Scenarios**:
    *   **Scenario 1**: Request without a header. **Expected**: 401 Error.
    *   **Scenario 2**: Scripted burst of 150 requests in 10 seconds. **Expected**: Requests 101-150 receive 429.
*   **Affected Components**: `axum-api`, Redis, PostgreSQL.
*   **File References**: `src/api/auth.rs`, `src/api/rate_limit.rs`.
*   **Verification Steps**: Run `k6` load test script with and without valid tokens.

**Story Estimation**:
*   **Complexity**: Medium
*   **Dependencies**: PostgreSQL schema for API keys.

---

## Summary of Verification & Deployment

| Requirement | Verification Tool | Success Metric |
| :--- | :--- | :--- |
| **Performance** | `cargo bench` | < 500ms / 5 pages |
| **Security** | `cargo-audit`, `proptest` | 0 vulnerabilities, 0 unhandled panics |
| **Scalability** | `k6` + `kubectl` | Scale to 20 pods < 3 mins |
| **Operations** | Prometheus/Sentry | 100% trace coverage |
| **Client-Side** | Playwright | 1:1 Visual Fidelity |

### Implementation Order
1.  **US-NFR-002 (Security)**: Must be first to ensure the core is safe.
2.  **US-NFR-001 (Performance)**: Establish baseline core speed.
3.  **US-NFR-004 (Operations)**: Set up monitoring to track the next phases.
4.  **US-NFR-003 (Scalability)**: Configure infra for load.
5.  **US-NFR-006 (Auth)**: Secure the endpoints.
6.  **US-NFR-005 (Wasm)**: Optimize costs with client-side offloading.