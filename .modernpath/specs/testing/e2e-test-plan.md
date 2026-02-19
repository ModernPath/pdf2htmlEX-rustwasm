# E2E Test Plan: Oxidized Document Engine (ODE)

## 1. Project Overview
This test plan covers the End-to-End (E2E) validation of the **Oxidized Document Engine (ODE)**. The focus is on verifying the complete lifecycle of a document—from ingestion and Rust-based transformation to WebAssembly (Wasm) client-side rendering and UI integration.

---

## 2. Test Strategy
We will utilize **Playwright** for browser-based E2E flows and visual regression, and **Postman/Newman** (or Rust-based integration tests) for API flow validation. Testing will span the `dev` and `staging` environments hosted on AWS EKS.

---

## 3. Test Cases

### TC-001: Standard Server-Side PDF to HTML Conversion Flow
- **Description**: Verify a standard PDF can be uploaded via API and converted to high-fidelity HTML.
- **Preconditions**: API Service is running; PostgreSQL and Redis are healthy.
- **Test Steps**:
    1. POST a multipart/form-data request to `/api/v1/convert` with `sample_invoice.pdf`.
    2. Capture the `job_id` from the JSON response.
    3. Poll `/api/v1/jobs/{job_id}` until status is `COMPLETED`.
    4. GET `/api/v1/documents/{job_id}/render`.
- **Expected Result**: HTTP 200; Response body contains semantic HTML5; CSS includes embedded fonts matching the original PDF.
- **Actual Files**: `src/api/conversion.rs`, `src/engine/core.rs`

### TC-002: Client-Side Wasm Transformation Flow
- **Description**: Verify the React frontend can process a PDF locally using the ODE Wasm module.
- **Preconditions**: Wasm module is built and served via the frontend assets.
- **Test Steps**:
    1. Navigate to the ODE Web Portal.
    2. Toggle "Client-Side Processing" mode.
    3. Upload `brochure.pdf` via the Radix UI upload component.
    4. Monitor the browser console for Wasm initialization logs.
- **Expected Result**: Document renders in the `<DocumentViewer />` component without a backend conversion call; UI remains responsive during processing.
- **Actual Files**: `frontend/src/hooks/useWasmConverter.ts`, `src/wasm_bridge/lib.rs`

### TC-003: Visual Regression - Layout Fidelity
- **Description**: Ensure the rendered HTML is pixel-perfect compared to the source PDF.
- **Preconditions**: Playwright is configured with visual comparison snapshots.
- **Test Steps**:
    1. Run Playwright test: `npx playwright test visual-fidelity.spec.ts`.
    2. The test uploads `complex_layout.pdf` (containing overlapping SVGs and multi-column text).
    3. Capture a screenshot of the rendered HTML in Chromium.
    4. Compare against the baseline `complex_layout.png`.
- **Expected Result**: Image mismatch is less than 0.05% (ignoring anti-aliasing).
- **Actual Files**: `e2e/visual-fidelity.spec.ts`

### TC-004: Interactive Element Preservation (Hyperlinks & Forms)
- **Description**: Verify that interactive elements in the PDF are converted to functional HTML elements.
- **Preconditions**: PDF with internal/external links and form fields.
- **Test Steps**:
    1. Convert `interactive_form.pdf`.
    2. Open the resulting HTML in the browser.
    3. Click an external link (e.g., `https://example.com`).
    4. Focus on an `<input>` field converted from a PDF form.
- **Expected Result**: Link opens in a new tab; Input field accepts text entry and maintains position relative to background text.
- **Actual Files**: `src/engine/dom_generator.rs`, `frontend/src/components/InteractiveLayer.tsx`

### TC-005: High-Throughput Batch Processing
- **Description**: Verify the system handles multiple concurrent conversion requests via Redis Task Queue.
- **Preconditions**: Redis is running; 5 Worker pods are active in EKS.
- **Test Steps**:
    1. Use a script to POST 50 conversion requests simultaneously.
    2. Monitor Grafana dashboard for "Active Workers" and "Queue Depth".
    3. Verify all 50 jobs reach `COMPLETED` status.
- **Expected Result**: No 5xx errors from Axum; Redis successfully distributes tasks; All metadata is correctly persisted in PostgreSQL.
- **Actual Files**: `src/api/queue.rs`, `src/worker/main.rs`

### TC-006: Accessibility Compliance (WCAG 2.1 AA)
- **Description**: Verify the output HTML follows semantic structure for screen readers.
- **Preconditions**: `axe-core` integrated with Playwright.
- **Test Steps**:
    1. Convert `academic_paper.pdf`.
    2. Run `axe` accessibility scan on the rendered output.
- **Expected Result**: Zero "Critical" or "Serious" violations; `<h1>` through `<h6>` tags reflect the PDF outline; Images have `alt` tags if present in PDF metadata.
- **Actual Files**: `src/engine/semantic_parser.rs`

### TC-007: Memory Safety & Malicious File Handling
- **Description**: Ensure the Rust engine gracefully handles "Zip Bomb" style PDFs or corrupted headers.
- **Preconditions**: Sentry is configured to capture panic events.
- **Test Steps**:
    1. Upload `malformed_header.pdf`.
    2. Upload `deep_nesting_bomb.pdf`.
- **Expected Result**: API returns `422 Unprocessable Entity`; Rust core does not panic; Memory usage of the worker pod does not exceed 512MB limit (OOMKilled check).
- **Actual Files**: `src/engine/parser/mod.rs`

### TC-008: Authentication & Protected Document Access
- **Description**: Verify that converted documents are only accessible to authorized users.
- **Preconditions**: JWT-based authentication is enabled in Axum.
- **Test Steps**:
    1. Convert a document as `User_A`.
    2. Attempt to GET the document render URL using `User_B`'s token.
- **Expected Result**: HTTP 403 Forbidden.
- **Actual Files**: `src/api/auth.rs`, `src/api/middleware.rs`

### TC-009: Observability & Tracing (OpenTelemetry)
- **Description**: Verify that a conversion request generates a complete trace across API and Worker.
- **Preconditions**: Jaeger/Tempo is running in the dev environment.
- **Test Steps**:
    1. Trigger a conversion job.
    2. Search for the `trace_id` in the Jaeger UI.
- **Expected Result**: Trace shows spans for: `http_request` -> `redis_push` -> `worker_process` -> `postgres_update`.
- **Actual Files**: `src/telemetry.rs`

### TC-010: Resilience - Redis Connection Failure
- **Description**: Verify the API handles transient Redis failures using retries.
- **Preconditions**: Access to `kubectl` to simulate network partitions.
- **Test Steps**:
    1. Start a conversion request.
    2. Immediately scale Redis replicas to 0.
    3. Wait 5 seconds and scale Redis back to 1.
- **Expected Result**: Axum API uses exponential backoff; Job is eventually queued without user-facing error.
- **Actual Files**: `src/infrastructure/redis_client.rs`

---

## 4. Test Data
| Data Type | File Name | Purpose |
| :--- | :--- | :--- |
| **Standard** | `invoice_v1.pdf` | Happy path testing. |
| **Complex** | `blueprint_vector.pdf` | SVG rendering and layout fidelity. |
| **Edge Case** | `zero_byte.pdf` | Error handling for empty files. |
| **Edge Case** | `password_protected.pdf` | Verification of encryption handling. |
| **Large** | `100mb_manual.pdf` | Performance and timeout testing. |

---

## 5. Verification Checklist

### Acceptance Criteria Verification
- [ ] **AC-01**: Conversion of a 10-page PDF completes in < 2 seconds (Server-side).
- [ ] **AC-02**: Wasm module size is < 5MB (compressed) for fast browser loading.
- [ ] **AC-03**: Rendered HTML passes WCAG 2.1 AA automated scans.
- [ ] **AC-04**: All API endpoints return standardized JSON error objects on failure.

### Implementation Readiness
- [ ] **Dependencies**: Ensure `cargo-wasm` and `wasm-pack` are installed in CI.
- [ ] **Environment**: Terraform has provisioned the S3 buckets for document storage.
- [ ] **Mocking**: Redis mocks are available for local unit testing.

### Verification Steps (Post-Implementation)
1. Execute `cargo test` to verify unit-level memory safety.
2. Run `npx playwright test` to validate E2E browser flows.
3. Check **Sentry** for any captured panics during the "Malicious File" test suite.
4. Validate **Prometheus** metrics for `document_conversion_duration_seconds` histograms.