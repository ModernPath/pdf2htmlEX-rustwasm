# Integration Test Plan: Oxidized Document Engine (ODE)

This document outlines the integration testing strategy for the Oxidized Document Engine (ODE). The focus is on validating the interaction between the Rust-based conversion core, the Axum API, the asynchronous task queue (Redis), and the persistent metadata store (PostgreSQL).

---

## 1. Project Overview & Scope
The goal of these integration tests is to ensure that the individual components of the ODE work together to transform PDF documents into high-fidelity HTML/CSS. We are testing the "seams" between the API, the Worker, and the Storage layers.

---

## 2. Test Cases

### TC-001: End-to-End Job Lifecycle (API -> Redis -> Worker)
- **Description**: Verify that a PDF upload successfully triggers a background job and produces HTML output.
- **Preconditions**: API, Redis, and Worker services are running in the `test` environment.
- **Test Steps**:
    1. POST a valid PDF file to `/api/v1/convert`.
    2. Capture the `job_id` from the response.
    3. Poll `/api/v1/jobs/{job_id}` until status is `COMPLETED`.
    4. GET the output URL provided in the job metadata.
- **Expected Result**: Job status transitions `PENDING` -> `PROCESSING` -> `COMPLETED`. Output URL returns a valid HTML document with status 200.
- **Actual Files**: `src/api/handlers/upload.rs`, `src/worker/processor.rs`, `tests/integration/e2e_flow.rs`

### TC-002: Metadata Persistence (API -> PostgreSQL)
- **Description**: Ensure job parameters and status are correctly mirrored in the database.
- **Preconditions**: PostgreSQL database is initialized with the latest migrations.
- **Test Steps**:
    1. Submit a conversion job with specific options (e.g., `--embed-fonts false`).
    2. Directly query the `jobs` table in PostgreSQL using the `job_id`.
    3. Compare the `options` JSONB column with the submitted parameters.
- **Expected Result**: Database record matches the submitted API payload exactly.
- **Actual Files**: `src/db/models.rs`, `src/api/state.rs`, `tests/integration/db_metadata.rs`

### TC-003: Task Queue Reliability (Worker -> Redis)
- **Description**: Verify that the worker correctly ACKs tasks from Redis and handles worker crashes.
- **Preconditions**: Redis is populated with 1 dummy task.
- **Test Steps**:
    1. Start the worker and intercept the processing loop.
    2. Kill the worker process mid-conversion.
    3. Restart the worker.
- **Expected Result**: The task should be re-queued (visibility timeout) and picked up again for processing, not lost.
- **Actual Files**: `src/queue/redis_client.rs`, `src/worker/main.rs`, `tests/integration/queue_resilience.rs`

### TC-004: Wasm Module Integration (React -> Wasm Core)
- **Description**: Validate that the React viewer can successfully load and execute the Wasm conversion core for client-side previews.
- **Preconditions**: Wasm binaries are compiled and available in the public assets folder.
- **Test Steps**:
    1. Initialize the `DocumentViewer` React component.
    2. Pass a Blob of a sample PDF to the component.
    3. Monitor the `onConversionComplete` callback.
- **Expected Result**: Wasm module initializes without memory errors and returns a DOM fragment.
- **Actual Files**: `ui/src/components/Viewer.tsx`, `ui/src/wasm/ode_core.wasm`, `ui/tests/wasm_integration.spec.ts`

### TC-005: Cache Hit/Miss Logic (API -> Redis Cache)
- **Description**: Ensure that identical PDF hashes return cached HTML instead of re-processing.
- **Preconditions**: A PDF has already been processed once.
- **Test Steps**:
    1. Submit PDF "A" and wait for completion.
    2. Submit the exact same PDF "A" again.
    3. Check the `X-Cache-Hit` header in the response or the `processed_at` timestamp.
- **Expected Result**: The second request returns immediately with a `CACHE_HIT` status; no new worker task is generated.
- **Actual Files**: `src/api/middleware/cache.rs`, `tests/integration/caching.rs`

### TC-006: Engine Error Propagation (Rust Core -> API)
- **Description**: Verify that internal Rust engine errors (e.g., password-protected PDF) are correctly mapped to HTTP status codes.
- **Preconditions**: An encrypted PDF file.
- **Test Steps**:
    1. Upload a password-protected PDF without providing a password.
    2. Inspect the API response body and status code.
- **Expected Result**: API returns `422 Unprocessable Entity` with a JSON body containing `error_code: "ENCRYPTED_PDF"`.
- **Actual Files**: `src/engine/error.rs`, `src/api/error_mapping.rs`, `tests/integration/error_handling.rs`

### TC-007: Telemetry Correlation (Axum -> OpenTelemetry)
- **Description**: Ensure a single `trace_id` spans from the API request to the background worker processing.
- **Preconditions**: Jaeger or OTel collector is running.
- **Test Steps**:
    1. Submit a job.
    2. Retrieve the `trace_id` from the API response headers.
    3. Query the OTel collector for all spans associated with that `trace_id`.
- **Expected Result**: At least two spans exist: one for the HTTP POST and one for the Worker processing logic.
- **Actual Files**: `src/telemetry/mod.rs`, `tests/integration/tracing.rs`

### TC-008: Resource Limit Enforcement (Tokio Runtime)
- **Description**: Ensure that the system handles high-concurrency PDF processing without dropping API requests.
- **Preconditions**: Set worker concurrency limit to 2.
- **Test Steps**:
    1. Submit 10 large PDF jobs simultaneously.
    2. Monitor API responsiveness (health check endpoint) while workers are saturated.
- **Expected Result**: API remains responsive (latency < 200ms) while jobs are queued in Redis.
- **Actual Files**: `src/api/main.rs`, `src/worker/config.rs`, `tests/integration/load_shedding.rs`

### TC-009: SVG/Font Asset Linking (Worker -> S3/Storage)
- **Description**: Verify that converted HTML correctly references extracted assets (fonts/images) in storage.
- **Preconditions**: S3-compatible storage (LocalStack) is configured.
- **Test Steps**:
    1. Convert a PDF containing custom fonts and images.
    2. Parse the output HTML.
    3. Verify that `<img>` and `@font-face` URLs are reachable and return the correct binary data.
- **Expected Result**: All linked assets in the HTML return 200 OK from the storage service.
- **Actual Files**: `src/storage/s3.rs`, `src/engine/html_writer.rs`, `tests/integration/asset_links.rs`

### TC-010: Database Transaction Rollback (DB -> API)
- **Description**: Ensure that if the Redis task creation fails, the PostgreSQL metadata entry is rolled back.
- **Preconditions**: Redis is intentionally taken offline.
- **Test Steps**:
    1. Attempt to upload a PDF.
    2. API should return a 500 error.
    3. Check PostgreSQL for the `job_id`.
- **Expected Result**: No record should exist in PostgreSQL for the failed job (Atomic transaction).
- **Actual Files**: `src/api/services/job_service.rs`, `tests/integration/transactions.rs`

---

## 3. Test Data

### Sample Data
- `standard_text.pdf`: 5-page document with standard fonts (Arial/Times).
- `complex_vector.pdf`: Document with heavy SVG and path data.
- `multi_byte_chars.pdf`: PDF containing CJK (Chinese, Japanese, Korean) characters to test font embedding.

### Edge Cases
- `zero_byte.pdf`: Empty file.
- `corrupt_header.pdf`: PDF with invalid magic bytes.
- `huge_dimensions.pdf`: PDF with a page size of 200x200 inches.
- `deeply_nested_layers.pdf`: PDF with >100 nested OCG layers.

### Error Scenarios
- **Network Timeout**: Simulate a 30s delay between API and Redis.
- **Disk Full**: Run the worker on a partition with <10MB space.
- **Auth Failure**: Submit jobs with expired JWT tokens.

---

## 4. Verification Checklist & Acceptance Criteria

### Implementation Verification
- [ ] **TC-001 to TC-10** are implemented as automated tests in the `tests/integration/` directory.
- [ ] Tests use `testcontainers-rs` to spin up ephemeral PostgreSQL and Redis instances.
- [ ] Code coverage for integration paths is >80%.

### Acceptance Criteria
- [ ] **Atomic Operations**: No job metadata exists in PostgreSQL without a corresponding task in Redis (and vice versa).
- [ ] **Memory Safety**: Worker process does not exceed 1GB RSS during a 100MB PDF conversion (verified via `/metrics`).
- [ ] **Visual Fidelity**: Visual regression (Playwright) shows <1% pixel difference between original PDF and rendered HTML.
- [ ] **Latency**: API job submission response time is <100ms (excluding file upload time).

### Dependencies
1. **Pre-requisite**: Rust Core Engine (Wasm/Native) must pass all unit tests.
2. **Pre-requisite**: Database schema migrations must be finalized.
3. **Post-requisite**: Performance benchmarks (k6) to be run after integration test suite passes.

### Verification Steps (Execution)
1. Run `cargo test --test '*' --features integration`.
2. Observe logs for OpenTelemetry span generation.
3. Check `target/debug/test-results` for visual regression diffs.
4. Verify Prometheus metrics at `localhost:9091/metrics` during test execution to ensure counters (e.g., `jobs_processed_total`) increment correctly.