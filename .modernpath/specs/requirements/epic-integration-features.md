This document outlines the **User Stories** for the **Integration Features Epic** of the Oxidized Document Engine (ODE). These stories focus on enabling external systems and developers to interact with the engine via APIs, Webhooks, and Client-side Wasm modules.

---

## Epic: Integration Features
**Goal**: To provide a robust, secure, and developer-friendly set of interfaces for programmatic document transformation.

### US-001: RESTful API Authentication (API Keys)
**As a** Backend Developer  
**I want to** generate and manage scoped API keys  
**So that** I can securely authenticate my external application's requests to the ODE service.

*   **Acceptance Criteria**:
    *   [ ] Users can generate API keys via the React dashboard.
    *   [ ] Keys must be hashed using Argon2 before being stored in PostgreSQL.
    *   [ ] The system must support "Read-only" and "Full-access" scopes.
    *   [ ] API requests must be rejected with a `401 Unauthorized` if the `X-API-KEY` header is missing or invalid.
*   **Test Scenarios**:
    *   *Scenario*: Submit a conversion request with a revoked key. *Expected*: `401 Unauthorized`.
    *   *Scenario*: Submit a request with a "Read-only" key to a "Delete" endpoint. *Expected*: `403 Forbidden`.
*   **Affected Components**: `ode-api-gateway` (Axum), `postgres-db` (Metadata), `ode-dashboard` (React).
*   **File References**: `src/auth/middleware.rs`, `src/models/api_key.rs`, `src/db/migrations/001_create_api_keys.sql`.
*   **Complexity**: Medium
*   **Dependencies**: Database schema for users/organizations must exist.

---

### US-002: Asynchronous Conversion Endpoint
**As a** Frontend Engineer  
**I want to** submit a PDF via a POST request and receive a Job ID  
**So that** my application can process large documents without timing out the HTTP connection.

*   **Acceptance Criteria**:
    *   [ ] Endpoint `POST /v1/convert` accepts `multipart/form-data` (PDF file) or a JSON body with a URL.
    *   [ ] Returns a `202 Accepted` status with a JSON payload containing a `job_id`.
    *   [ ] The job metadata is persisted in PostgreSQL and the task is pushed to the Redis queue.
    *   [ ] Input validation ensures only valid PDF magic numbers are accepted.
*   **Test Scenarios**:
    *   *Scenario*: Post a 50MB PDF. *Expected*: Immediate `202 Accepted` and a valid UUID `job_id`.
    *   *Scenario*: Post a text file renamed to `.pdf`. *Expected*: `400 Bad Request` (Invalid File Format).
*   **Affected Components**: `ode-api` (Axum/Tokio), `redis-queue`, `ode-worker` (Rust).
*   **File References**: `src/routes/convert.rs`, `src/queue/producer.rs`.
*   **Complexity**: Medium
*   **Dependencies**: Redis infrastructure must be provisioned via Terraform.

---

### US-003: Webhook Notifications for Job Completion
**As an** Enterprise Content Manager  
**I want to** register a webhook URL to receive a POST notification when a conversion is finished  
**So that** my downstream CMS can automatically ingest the resulting HTML.

*   **Acceptance Criteria**:
    *   [ ] Users can provide a `webhook_url` in the initial conversion request.
    *   [ ] Upon job completion (Success or Failure), the system sends a POST request with a signed payload.
    *   [ ] Payload must include `job_id`, `status`, `output_url`, and `timestamp`.
    *   [ ] Implements a retry strategy (exponential backoff) for failed webhook deliveries (e.g., 5xx errors).
*   **Test Scenarios**:
    *   *Scenario*: Complete a job with a valid webhook URL. *Expected*: Target URL receives a POST with `status: "completed"`.
    *   *Scenario*: Target URL is down. *Expected*: System logs a retry attempt in Prometheus/Sentry.
*   **Affected Components**: `ode-worker`, `webhook-dispatcher` (Tokio-based service).
*   **File References**: `src/services/webhooks.rs`, `src/models/webhook_payload.rs`.
*   **Complexity**: High
*   **Dependencies**: US-002 (Async Conversion).

---

### US-004: Client-Side Wasm Integration Library
**As a** Frontend Developer  
**I want to** import the ODE engine as a WebAssembly module into my React project  
**So that** I can perform PDF-to-HTML conversions locally in the user's browser for maximum privacy.

*   **Acceptance Criteria**:
    *   [ ] Provide a `@oxidized-doc/engine-wasm` NPM package.
    *   [ ] The Wasm module must expose a `convert_pdf(buffer: Uint8Array): string` function.
    *   [ ] TypeScript definitions (`.d.ts`) must be included for all exported functions.
    *   [ ] Memory usage must be capped to prevent browser tab crashes on complex PDFs.
*   **Test Scenarios**:
    *   *Scenario*: Load the Wasm module in a Chrome environment and convert a 1-page PDF. *Expected*: Valid HTML string returned in < 500ms.
    *   *Scenario*: Run conversion in a Web Worker. *Expected*: UI remains responsive during processing.
*   **Affected Components**: `ode-core` (Rust), `wasm-pack` build pipeline, `ode-viewer` (React component).
*   **File References**: `crate/ode-wasm/src/lib.rs`, `packages/engine-wasm/package.json`.
*   **Complexity**: High
*   **Dependencies**: Rust-to-Wasm toolchain (wasm-bindgen).

---

### US-005: S3 Presigned URL Integration
**As a** Backend Developer  
**I want to** provide presigned S3 URLs for input and output  
**So that** the ODE API doesn't become a bottleneck for binary data transfer.

*   **Acceptance Criteria**:
    *   [ ] API supports `input_s3_url` in the request body.
    *   [ ] Upon completion, the engine generates a presigned GET URL for the resulting HTML/CSS assets.
    *   [ ] Presigned URLs must have a configurable expiration (default 1 hour).
    *   [ ] IAM roles for the EKS pods must be restricted to specific S3 buckets.
*   **Test Scenarios**:
    *   *Scenario*: Submit job with a private S3 URL. *Expected*: Worker downloads file using IAM role permissions.
    *   *Scenario*: Access output URL after 61 minutes. *Expected*: `403 Forbidden` from AWS S3.
*   **Affected Components**: `ode-worker`, `aws-sdk-rust`.
*   **File References**: `src/storage/s3.rs`, `terraform/iam.tf`.
*   **Complexity**: Medium
*   **Dependencies**: AWS Infrastructure (S3 Buckets).

---

### US-006: OpenAPI 3.0 Documentation (Swagger)
**As a** Third-party Developer  
**I want to** access an interactive API documentation page  
**So that** I can test endpoints and understand the data schema without reading source code.

*   **Acceptance Criteria**:
    *   [ ] Automatically generate OpenAPI spec from Axum routes using `utoipa`.
    *   [ ] Serve Swagger UI at `/docs`.
    *   [ ] All schemas (Job, Error, APIKey) must be fully documented with examples.
    *   [ ] The "Try it out" feature must work with a test API key.
*   **Test Scenarios**:
    *   *Scenario*: Navigate to `/docs`. *Expected*: Swagger UI renders with all v1 endpoints.
    *   *Scenario*: Download `openapi.json`. *Expected*: Valid JSON that passes OpenAPI linter.
*   **Affected Components**: `ode-api`.
*   **File References**: `src/docs/openapi.rs`, `src/main.rs`.
*   **Complexity**: Low
*   **Dependencies**: None.

---

## Story Estimation & Prioritization

| ID | Title | Complexity | Priority | Dependencies |
| :--- | :--- | :--- | :--- | :--- |
| **US-001** | API Authentication | Medium | P0 | None |
| **US-002** | Async Conversion API | Medium | P0 | Redis |
| **US-005** | S3 Presigned URLs | Medium | P1 | AWS S3 |
| **US-003** | Webhook Notifications | High | P1 | US-002 |
| **US-004** | Client-Side Wasm | High | P2 | ode-core |
| **US-006** | OpenAPI Docs | Low | P2 | US-001, US-002 |

---

## Verification Plan

### Automated Testing
1.  **Integration Tests**: Run `cargo test --test integration_api` to verify the Axum-to-Postgres flow.
2.  **Property-based Testing**: Use `proptest` in Rust to send malformed JSON payloads to the API to ensure no panics occur.
3.  **Visual Regression**: Use Playwright to verify that the HTML output generated via the API matches the source PDF visually.

### Manual Verification Steps
1.  **Key Generation**: Log into the Dashboard, create a key, and use it in a `curl` command.
2.  **Webhook Loop**: Use a service like Webhook.site to verify that the worker sends the correct payload upon job completion.
3.  **Wasm Load**: Import the generated NPM package into a sample Vite project and verify the `convert_pdf` function returns a valid DOM string.

### Monitoring & Observability
-   **Grafana**: Monitor `http_requests_total` and `conversion_job_duration_seconds`.
-   **Sentry**: Verify that API `500` errors are captured with full stack traces from the Axum handlers.
-   **OpenTelemetry**: Trace a single request from the API Gateway -> Redis -> Worker -> S3.