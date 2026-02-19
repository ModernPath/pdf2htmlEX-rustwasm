# API Test Plan: Oxidized Document Engine (ODE)

This test plan defines the suite of automated and manual tests required to validate the **Oxidized Document Engine (ODE)** API. The focus is on the Axum-based REST API, job orchestration via Redis/Postgres, and the fidelity of the Rust-to-Wasm conversion engine.

---

## 1. Project Overview
The ODE provides a high-performance API for converting PDF documents into semantic HTML5/Wasm. It utilizes an asynchronous architecture (Tokio) to handle high-throughput batch processing.

## 2. Test Cases

### TC-001: Successful Synchronous Job Submission (Small PDF)
- **Description**: Verify that a valid, small PDF (<5MB) can be uploaded and queued for processing.
- **Preconditions**: API service is running; Redis and PostgreSQL are reachable.
- **Test Steps**:
    1. Send `POST /v1/jobs` with `multipart/form-data` containing a 1-page PDF.
    2. Include header `X-API-Key: <valid_key>`.
- **Expected Result**: 
    - HTTP 202 Accepted.
    - JSON Response contains a valid UUID `job_id`.
    - Database record created in `jobs` table with status `pending`.
- **Actual Files**: `src/api/handlers/jobs.rs`, `src/models/job.rs`

### TC-002: Job Status Polling Lifecycle
- **Description**: Verify the transition of a job from `pending` -> `processing` -> `completed`.
- **Preconditions**: Job submitted via TC-001.
- **Test Steps**:
    1. Poll `GET /v1/jobs/{job_id}` every 500ms.
- **Expected Result**:
    - Initial status: `pending`.
    - Intermediate status: `processing`.
    - Final status: `completed`.
    - Response includes `completed_at` timestamp and `output_url`.
- **Actual Files**: `src/api/handlers/jobs.rs`, `src/engine/worker.rs`

### TC-003: Retrieval of Conversion Output (Wasm/HTML Bundle)
- **Description**: Ensure the API correctly serves the generated conversion artifacts.
- **Preconditions**: Job status is `completed`.
- **Test Steps**:
    1. Send `GET /v1/jobs/{job_id}/output`.
- **Expected Result**:
    - HTTP 200 OK.
    - Content-Type is `application/zip` or `text/html`.
    - Payload contains valid HTML5 and `.wasm` binaries.
- **Actual Files**: `src/api/handlers/outputs.rs`

### TC-004: Error Handling - Password Protected PDF
- **Description**: Verify the engine identifies and reports encrypted PDFs that require a password.
- **Preconditions**: A PDF encrypted with AES-256 is available.
- **Test Steps**:
    1. Send `POST /v1/jobs` with the encrypted PDF.
- **Expected Result**:
    - HTTP 202 Accepted (Job is queued).
    - `GET /v1/jobs/{job_id}` eventually returns status `failed`.
    - `error_code` in JSON is `ENCRYPTED_PDF_NO_PASSWORD`.
- **Actual Files**: `src/engine/pdf_parser.rs`, `src/api/errors.rs`

### TC-005: Boundary Test - Maximum File Size Limit
- **Description**: Verify the API rejects files exceeding the configured limit (e.g., 100MB).
- **Preconditions**: `MAX_UPLOAD_SIZE` set to 100MB in `.env`.
- **Test Steps**:
    1. Attempt `POST /v1/jobs` with a 105MB PDF file.
- **Expected Result**:
    - HTTP 413 Payload Too Large.
    - JSON error message: "File exceeds maximum allowed size of 100MB."
- **Actual Files**: `src/api/middleware/size_limit.rs`

### TC-006: Input Validation - Unsupported File Format
- **Description**: Ensure the API rejects non-PDF files.
- **Preconditions**: A `.docx` or `.txt` file.
- **Test Steps**:
    1. Send `POST /v1/jobs` with `document.txt`.
- **Expected Result**:
    - HTTP 400 Bad Request.
    - JSON error message: "Invalid file type. Only application/pdf is supported."
- **Actual Files**: `src/api/handlers/jobs.rs`

### TC-007: Concurrent Job Processing (Stress Test)
- **Description**: Verify the system handles 50 concurrent conversion requests without memory leaks or deadlocks.
- **Preconditions**: Kubernetes pod limits set; Prometheus monitoring active.
- **Test Steps**:
    1. Use a load testing tool (e.g., `k6` or `locust`) to fire 50 POST requests simultaneously.
- **Expected Result**:
    - All jobs receive HTTP 202.
    - Redis queue processes jobs sequentially based on worker availability.
    - No 5xx errors observed.
- **Actual Files**: `src/engine/worker.rs`, `src/api/main.rs`

### TC-008: Job Cancellation and Resource Cleanup
- **Description**: Verify that deleting a job stops processing and cleans up storage.
- **Preconditions**: A job is currently in `processing` status.
- **Test Steps**:
    1. Send `DELETE /v1/jobs/{job_id}`.
    2. Check Redis for the task ID.
    3. Check S3/Local storage for temporary artifacts.
- **Expected Result**:
    - HTTP 204 No Content.
    - Task removed from Redis queue.
    - Temporary files deleted from `/tmp/ode/`.
- **Actual Files**: `src/api/handlers/jobs.rs`, `src/storage/mod.rs`

### TC-009: Health Check and Metrics Endpoint
- **Description**: Verify monitoring endpoints provide accurate system status.
- **Preconditions**: System is under partial load.
- **Test Steps**:
    1. Send `GET /health`.
    2. Send `GET /metrics`.
- **Expected Result**:
    - `/health` returns `{"status": "up", "db": "connected", "redis": "connected"}`.
    - `/metrics` returns Prometheus formatted strings including `ode_jobs_processed_total`.
- **Actual Files**: `src/api/handlers/monitoring.rs`

### TC-010: Property-Based Testing (Fuzzing) for Conversion Options
- **Description**: Use `proptest` logic to ensure the API handles arbitrary JSON configuration options without crashing.
- **Preconditions**: Access to `cargo test`.
- **Test Steps**:
    1. Execute `POST /v1/jobs` with a JSON body containing randomized keys and values in the `options` field.
- **Expected Result**:
    - API returns 400 for invalid schemas.
    - Engine never panics (Rust safety).
- **Actual Files**: `tests/property_tests.rs`, `src/models/options.rs`

---

## 3. Test Data

### Sample Files
| File Name | Description | Purpose |
| :--- | :--- | :--- |
| `simple_1p.pdf` | 1-page text-only PDF | Baseline functional test |
| `vector_complex.pdf` | PDF with complex SVG paths | Visual fidelity test |
| `font_heavy.pdf` | Uses non-standard embedded fonts | Wasm font-rendering test |
| `large_report.pdf` | 50MB+, 200+ pages | Performance/Timeout test |
| `protected.pdf` | Password required | Error handling test |

### Edge Cases
- **Zero-byte PDF**: Should return 400 Bad Request.
- **Corrupt PDF Header**: Should return 422 Unprocessable Entity after engine analysis.
- **PDF with 0 pages**: Should handle gracefully without engine panic.

---

## 4. Verification Checklist

- [ ] **Acceptance Criteria**: All 2xx responses return valid JSON schemas as defined in OpenAPI spec.
- [ ] **Acceptance Criteria**: Conversion fidelity (visual) matches original PDF within 98% pixel similarity (Playwright Visual Regression).
- [ ] **Edge Cases**: System handles Redis disconnection by retrying job status updates (Exponential Backoff).
- [ ] **Error Handling**: All error responses include a unique `request_id` for Sentry log correlation.
- [ ] **Performance**: Average conversion time for `simple_1p.pdf` is < 800ms.

---

## 5. Implementation & Dependencies

### Dependencies
1. **Pre-requisite**: PostgreSQL schema migration `20231027_init_jobs.sql` must be applied.
2. **Pre-requisite**: Redis instance must be available for the `tokio-retry` logic.
3. **Downstream**: React Document Viewer component depends on the output of `TC-003`.

### Verification Steps (Post-Implementation)
1. Run `cargo test` to execute unit tests in `src/api/handlers/`.
2. Deploy to `dev` environment using GitHub Actions.
3. Run the Newman/Postman collection against the `dev` endpoint:
   ```bash
   newman run ./tests/collections/ode_api_tests.json -e ./tests/env/dev.json
   ```
4. Verify logs in CloudWatch/Sentry to ensure no hidden panics occurred during the test run.