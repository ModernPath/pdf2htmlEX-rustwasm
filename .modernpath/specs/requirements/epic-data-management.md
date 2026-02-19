This document outlines the detailed user stories for the **Data Management Epic** within the **Oxidized Document Engine (ODE)** project. The focus is on CRUD (Create, Read, Update, Delete) operations for the primary entities: **Conversion Jobs**, **Document Metadata**, and **Processing Profiles**.

---

## Epic: Data Management
**Goal**: Provide a robust, memory-safe, and high-performance API and UI for managing the lifecycle of document conversion tasks.

### US-001: Create Conversion Job (The "C" in CRUD)
**As a** Backend Developer  
**I want to** submit a PDF file and conversion parameters via a REST API  
**So that** the engine can queue the document for high-fidelity HTML/Wasm transformation.

*   **Acceptance Criteria**:
    *   [ ] API endpoint `POST /api/v1/jobs` accepts `multipart/form-data` (PDF file) and JSON metadata.
    *   [ ] System validates that the file is a valid PDF and under the size limit (default 50MB).
    *   [ ] A new record is created in PostgreSQL `jobs` table with status `PENDING`.
    *   [ ] A task is pushed to the Redis queue for the Rust worker to pick up.
    *   [ ] Returns a `201 Created` status with a unique `job_id` (UUID v4).
*   **Test Scenarios**:
    *   *Success*: Upload a valid 5-page PDF; verify `job_id` is returned and status in DB is `PENDING`.
    *   *Failure*: Upload a `.txt` file; verify `400 Bad Request` with error message "Invalid file type."
    *   *Failure*: Submit without required parameters; verify `422 Unprocessable Entity`.
*   **Affected Components**: `Axum API`, `PostgreSQL (Job Metadata)`, `Redis (Task Queue)`, `Rust Validator`.
*   **File References**:
    *   `src/api/handlers/jobs.rs` (New)
    *   `src/models/job.rs` (New)
    *   `migrations/001_create_jobs_table.sql`
*   **Story Estimation**:
    *   **Complexity**: Medium
    *   **Dependencies**: Database schema initialized, Redis connection pool configured.
*   **Verification Steps**:
    *   [ ] Run `cargo test api_job_creation`
    *   [ ] Use `Postman` or `curl` to POST a sample PDF and check the PostgreSQL `jobs` table.

---

### US-002: Retrieve Job Status and Metadata (The "R" in CRUD)
**As a** Frontend Engineer  
**I want to** poll or fetch the current status and metadata of a specific job  
**So that** I can update the UI progress bar and display conversion results to the user.

*   **Acceptance Criteria**:
    *   [ ] API endpoint `GET /api/v1/jobs/{job_id}` returns job details.
    *   [ ] Response includes: `job_id`, `status` (PENDING, PROCESSING, COMPLETED, FAILED), `created_at`, `completed_at`, and `output_url`.
    *   [ ] If the job failed, a `error_log` field provides a sanitized summary of the failure.
    *   [ ] Response time is < 50ms for indexed lookups in PostgreSQL.
*   **Test Scenarios**:
    *   *Success*: Fetch an existing job; verify JSON structure matches the schema.
    *   *Failure*: Fetch a non-existent UUID; verify `404 Not Found`.
*   **Affected Components**: `Axum API`, `PostgreSQL`, `React UI (Status Hook)`.
*   **File References**:
    *   `src/api/handlers/jobs.rs`
    *   `ui/src/hooks/useJobStatus.ts`
*   **Story Estimation**:
    *   **Complexity**: Low
    *   **Dependencies**: US-001 (Job Creation).
*   **Verification Steps**:
    *   [ ] Verify via `Playwright` that the UI displays "Processing" then "Completed" based on API response.

---

### US-003: List and Filter Conversion Jobs (The "R" in CRUD)
**As an** Enterprise Content Manager  
**I want to** view a paginated list of all conversion jobs with filtering options  
**So that** I can monitor system throughput and identify failed batches.

*   **Acceptance Criteria**:
    *   [ ] API endpoint `GET /api/v1/jobs` supports query parameters: `status`, `limit`, `offset`.
    *   [ ] UI table built with Radix UI and Tailwind CSS displays job history.
    *   [ ] UI supports WCAG 2.1 AA keyboard navigation for row selection.
    *   [ ] Results are ordered by `created_at` DESC by default.
*   **Test Scenarios**:
    *   *Success*: Request `?status=FAILED&limit=10`; verify only failed jobs are returned.
    *   *Performance*: Test with 10,000 records; verify pagination keeps response under 200ms.
*   **Affected Components**: `Axum API`, `React Dashboard Component`, `PostgreSQL Indexing`.
*   **File References**:
    *   `ui/src/components/JobTable.tsx`
    *   `src/db/queries/jobs.rs`
*   **Story Estimation**:
    *   **Complexity**: Medium
    *   **Dependencies**: Database indexing on `status` and `created_at`.
*   **Verification Steps**:
    *   [ ] [ ] Verify pagination controls in UI work correctly.
    *   [ ] [ ] Run `proptest` on the filtering logic to ensure no edge-case crashes.

---

### US-004: Update Job Configuration/Metadata (The "U" in CRUD)
**As a** Digital Archivist  
**I want to** update the "friendly name" or "priority" of a queued job  
**So that** I can better organize my archives or expedite critical documents.

*   **Acceptance Criteria**:
    *   [ ] API endpoint `PATCH /api/v1/jobs/{job_id}` allows updating `friendly_name` and `priority`.
    *   [ ] If `priority` is updated and job is still `PENDING`, the position in the Redis queue is adjusted (or re-queued).
    *   [ ] System prevents updates to `status` via this endpoint (status is system-managed).
    *   [ ] Returns `200 OK` with the updated object.
*   **Test Scenarios**:
    *   *Success*: Rename a job from "Doc1" to "Annual Report 2023"; verify DB update.
    *   *Constraint*: Attempt to update `status` to "COMPLETED" manually; verify `403 Forbidden` or field is ignored.
*   **Affected Components**: `Axum API`, `PostgreSQL`, `Redis (Priority Queue logic)`.
*   **File References**:
    *   `src/api/handlers/jobs.rs`
    *   `src/queue/priority_manager.rs`
*   **Story Estimation**:
    *   **Complexity**: Medium
    *   **Dependencies**: Redis ZSET implementation for priority queuing.
*   **Verification Steps**:
    *   [ ] [ ] Update a job name and immediately fetch it to verify persistence.
    *   [ ] [ ] Verify priority change moves the task forward in the Redis queue.

---

### US-005: Delete Job and Cleanup Assets (The "D" in CRUD)
**As a** System Administrator  
**I want to** delete a job and its associated metadata/files  
**So that** I can comply with data retention policies and free up storage.

*   **Acceptance Criteria**:
    *   [ ] API endpoint `DELETE /api/v1/jobs/{job_id}` removes the record from PostgreSQL.
    *   [ ] Associated temporary files on disk or S3 are deleted asynchronously.
    *   [ ] If the job is currently `PROCESSING`, the worker is signaled to abort (SIGTERM to Wasm/Rust process).
    *   [ ] Returns `204 No Content` on success.
*   **Test Scenarios**:
    *   *Success*: Delete a completed job; verify DB row is gone and S3 bucket is empty.
    *   *Safety*: Attempt to delete a job that doesn't exist; verify `404 Not Found`.
*   **Affected Components**: `Axum API`, `PostgreSQL`, `File Storage Service`, `Tokio Cancellation Tokens`.
*   **File References**:
    *   `src/api/handlers/jobs.rs`
    *   `src/services/storage.rs`
*   **Story Estimation**:
    *   **Complexity**: High (due to cleanup and process cancellation).
    *   **Dependencies**: Storage service abstraction layer.
*   **Verification Steps**:
    *   [ ] [ ] Check S3/Local storage logs to confirm file deletion.
    *   [ ] [ ] Verify DB referential integrity (no orphaned metadata).

---

### US-006: Manage Conversion Profiles (CRUD for Settings)
**As a** Frontend Engineer  
**I want to** create and manage "Conversion Profiles" (e.g., "High Quality," "Mobile Optimized")  
**So that** I can reuse complex settings (zoom, font-mapping, SVG-optimization) across multiple jobs.

*   **Acceptance Criteria**:
    *   [ ] CRUD endpoints for `/api/v1/profiles` (GET, POST, PUT, DELETE).
    *   [ ] Profile schema includes: `name`, `scale_factor`, `embed_fonts` (bool), `image_quality` (int).
    *   [ ] Profiles can be linked to a Job via `profile_id` during creation.
    *   [ ] UI provides a settings panel to manage these profiles using Radix UI Dialogs.
*   **Test Scenarios**:
    *   *Success*: Create a profile "FastDraft" with low image quality; use it in a job; verify output is low-res.
    *   *Validation*: Create a profile with `scale_factor: 0`; verify `400 Bad Request`.
*   **Affected Components**: `PostgreSQL (Profiles Table)`, `Axum API`, `React Settings UI`.
*   **File References**:
    *   `src/models/profile.rs`
    *   `ui/src/components/ProfileManager.tsx`
*   **Story Estimation**:
    *   **Complexity**: Medium
    *   **Dependencies**: US-001 (to support `profile_id` foreign key).
*   **Verification Steps**:
    *   [ ] [ ] Verify WCAG compliance on the Profile Management modal.
    *   [ ] [ ] Run integration test: Create Profile -> Create Job with Profile -> Verify Output.

---

## Summary of Technical Requirements for Implementation

### 1. Database Schema (PostgreSQL)
```sql
CREATE TABLE conversion_profiles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    settings JSONB NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE jobs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    profile_id UUID REFERENCES conversion_profiles(id),
    status VARCHAR(20) NOT NULL, -- PENDING, PROCESSING, COMPLETED, FAILED
    file_path TEXT NOT NULL,
    output_path TEXT,
    priority INT DEFAULT 0,
    error_log TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    completed_at TIMESTAMPTZ
);
```

### 2. Monitoring & Verification
- **Prometheus**: Track `job_creation_total`, `job_failure_total`, and `processing_duration_seconds`.
- **Sentry**: Capture any panics in the Rust Axum handlers during CRUD operations.
- **Playwright**: Visual regression testing for the Job Dashboard and Profile Modals.

### 3. Deployment Steps
1. Apply Terraform changes for RDS (PostgreSQL) and ElastiCache (Redis).
2. Run SQL migrations via `sqlx-cli` or custom migration runner.
3. Deploy Axum API to EKS.
4. Deploy React Frontend to CloudFront/S3.