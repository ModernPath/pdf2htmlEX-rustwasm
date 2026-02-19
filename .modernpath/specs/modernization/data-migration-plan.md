# Modernization Specification: Data Migration Plan
**Project**: Oxidized Document Engine (ODE) / rust-pdf2html  
**Area**: Schema Changes, Data Transformation Rules, and Migration Scripts

## 1. Source Analysis
The source system, `pdf2htmlEX`, is a stateless CLI utility. It does not utilize a relational database; instead, it relies on in-memory C++ structures and filesystem-based persistence.

*   **Key Business Logic**: 
    *   **PDF Parsing**: Orchestration layer scans fonts and page dimensions.
    *   **State Tracking**: The `HTMLRenderer` maintains a `GraphicsState` (transformations, clipping paths, colors).
    *   **Artifact Generation**: The `BackgroundRenderer` produces SVGs/Bitmaps, while the `HTMLRenderer` produces HTML/CSS and extracts fonts via FontForge.
*   **Data Structures**: 
    *   Transient C++ objects: `HTMLRenderer` state, `FontInfo` maps, and `Page` metadata.
    *   Output: Static files (HTML, CSS, WOFF/TTF, SVG, PNG).
*   **Integration Points**: 
    *   CLI arguments (input/output paths, rendering flags).
    *   External library calls (Poppler/PDF.js, Cairo, FontForge).

## 2. Target Architecture Mapping
The target architecture (ODE) moves from a stateless CLI to a **stateful, asynchronous service**.

| Source Pattern (pdf2htmlEX) | Target Pattern (ODE) | Technology Change |
| :--- | :--- | :--- |
| CLI Flags (`--zoom`, `--embed-css`) | **Job Configuration** | PostgreSQL (`configs` table) |
| In-memory Page Processing | **Task Queue** | Redis (Distributed Task Management) |
| Local Filesystem Output | **Object Storage** | AWS S3 + PostgreSQL Metadata |
| C++ `GraphicsState` | **Job Metadata** | PostgreSQL (`job_metadata` JSONB) |
| Console Logs | **Observability** | OpenTelemetry + PostgreSQL Logs |

### New Code Structure
- `src/models/`: Rust structs for PostgreSQL entities (using `sqlx` or `diesel`).
- `migrations/`: SQL files for schema evolution.
- `src/storage/`: Logic for persisting artifacts to S3/Local storage.
- `src/queue/`: Redis-backed task definitions.

## 3. Transformation Steps

### Phase 1: Metadata Schema Implementation
Define the core database schema to track the lifecycle of a document transformation.

- [ ] **Step 1**: Create the `jobs` table to replace CLI execution tracking.
- [ ] **Step 2**: Create the `assets` table to track generated HTML, CSS, and Fonts.
- **Source**: CLI Execution context → **Target**: `migrations/001_create_jobs.sql`

### Phase 2: Configuration Mapping
Convert legacy CLI flags into persistent configuration profiles.

- [ ] **Step 1**: Map `pdf2htmlEX` flags to a JSONB configuration schema.
- [ ] **Step 2**: Implement a "Default Profile" matching legacy standard behavior.
- **Source**: `src/main.cc` (Arg parsing) → **Target**: `src/models/config.rs`

### Phase 3: Data Ingestion & Storage Pipeline
- [ ] **Step 1**: Implement a migration script to move existing "Legacy Output" folders into the new S3-based structure.
- [ ] **Step 2**: Generate database records for legacy files to ensure the new ODE UI can "see" old conversions.
- **Source**: `output_dir/*` → **Target**: `src/services/storage.rs`

---

## 4. Database Schema Specification (PostgreSQL)

### Table: `jobs`
| Column | Type | Description |
| :--- | :--- | :--- |
| `id` | UUID (PK) | Unique identifier for the conversion task. |
| `status` | VARCHAR | `pending`, `processing`, `completed`, `failed`. |
| `source_url` | TEXT | S3 path to the original PDF. |
| `config_id` | UUID (FK) | Reference to the rendering settings used. |
| `metadata` | JSONB | Stores page count, PDF version, and title. |
| `created_at` | TIMESTAMP | Job submission time. |

### Table: `assets`
| Column | Type | Description |
| :--- | :--- | :--- |
| `id` | UUID (PK) | Unique asset ID. |
| `job_id` | UUID (FK) | Parent job. |
| `asset_type` | VARCHAR | `html`, `css`, `font`, `image`. |
| `s3_key` | TEXT | Location in AWS S3. |
| `checksum` | TEXT | SHA-256 for integrity verification. |

---

## 5. Risk Assessment

| Risk | Severity | Mitigation |
| :--- | :--- | :--- |
| **Schema Mismatch** | High | Use `sqlx` compile-time check for Rust models against the DB schema. |
| **Data Loss (Assets)** | Medium | Implement S3 Versioning and store SHA-256 checksums in PostgreSQL. |
| **Orphaned Files** | Low | Implement a cleanup worker that deletes S3 files not referenced in the `assets` table. |
| **Performance Bottleneck** | Medium | Use Redis for high-frequency status updates instead of writing to Postgres every second. |

**Rollback Plan**: 
1. Revert DB migrations using `sqlx-cli`.
2. Point the ODE API to a "Legacy Read-Only" S3 bucket containing original CLI outputs.

---

## 6. Verification Checklist & Test Strategy

### Acceptance Criteria
- [ ] **AC 1**: A PDF submitted via the Axum API creates a `job` record with status `pending`.
- [ ] **AC 2**: Completed jobs have a corresponding `asset` record for the `.html` file and all `.woff` fonts.
- [ ] **AC 3**: The `metadata` JSONB field correctly extracts the "Title" and "Author" from the PDF.
- [ ] **AC 4**: Checksums for migrated legacy files match their original source.

### Test Cases
- **TC 1: Schema Integrity**: Run `cargo test` to verify that Rust structs match the PostgreSQL table definitions.
- **TC 2: Migration Accuracy**: 
    - **Input**: A directory of files generated by legacy `pdf2htmlEX`.
    - **Action**: Run `ode-migrate --path ./legacy_out`.
    - **Expected**: Files exist in S3; `assets` table contains correct paths and job IDs.
- **TC 3: Concurrency Test**: Submit 100 simultaneous conversion jobs and verify Redis task distribution and DB connection pooling.

### Verification Steps
1. [ ] Run `sqlx migrate run` to apply schema.
2. [ ] Execute the `test_pdf_ingestion` integration test.
3. [ ] Query `SELECT count(*) FROM assets WHERE job_id = '...'` to ensure all 4-5 standard artifacts (HTML, CSS, 2+ Fonts) are registered.
4. [ ] Verify S3 object permissions allow the ODE React viewer to fetch assets via signed URLs.

## 7. File References
- **New Migration**: `migrations/20231027_initial_schema.sql`
- **New Data Model**: `src/models/job.rs`
- **New Storage Service**: `src/services/storage.rs`
- **Legacy Logic Reference**: `pdf2htmlEX/src/HTMLRenderer/HTMLRenderer.cc` (to ensure all generated file types are accounted for in the `asset_type` enum).