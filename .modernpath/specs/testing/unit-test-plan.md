# Unit Test Plan: Oxidized Document Engine (ODE)

This test plan defines the unit-level validation strategy for the `rust-pdf2html` core engine and its supporting modules. It focuses on isolating logic within the Rust core, WebAssembly bridge, and API layers to ensure memory safety, visual fidelity, and high-performance execution.

---

## 1. Project Overview & Scope
The goal of this unit test plan is to validate individual functions and modules within the **Oxidized Document Engine (ODE)**. Testing will leverage `cargo-test` for standard unit tests and `proptest` for property-based testing of complex PDF parsing logic.

**Target Modules:**
- `ode-core`: PDF parsing, layout calculation, and SVG generation.
- `ode-wasm`: JS-Rust serialization and memory management.
- `ode-api`: Axum handlers, job validation, and Redis/Postgres integration logic.
- `ode-ui`: React component logic and accessibility properties.

---

## 2. Test Cases

### TC-001: PDF Metadata Extraction Logic
- **Description**: Verify that the parser correctly extracts document metadata (Title, Author, Page Count) from a standard PDF buffer.
- **Preconditions**: A valid PDF byte buffer is loaded into memory.
- **Test Steps**:
    1. Initialize `PdfDocument::new(buffer)`.
    2. Call `document.get_metadata()`.
    3. Assert the returned `Metadata` struct fields.
- **Expected Result**: Metadata matches the source PDF; Page count is exactly 5.
- **Actual Files**: `src/engine/pdf_parser.rs`, `src/engine/models.rs`

### TC-002: Font Mapping and Substitution
- **Description**: Ensure that non-standard fonts are correctly mapped to fallback web-safe fonts or embedded CSS @font-face rules.
- **Preconditions**: A PDF containing a proprietary font (e.g., "Corporate-Bold") is used.
- **Test Steps**:
    1. Pass the font object to `FontManager::resolve_font()`.
    2. Check the returned `CssRule`.
- **Expected Result**: The system returns a fallback to `sans-serif` and generates a valid Data-URI for the embedded font.
- **Actual Files**: `src/engine/fonts.rs`

### TC-003: SVG Path Generation (Vector Graphics)
- **Description**: Validate that PDF vector paths are accurately converted to SVG path strings.
- **Preconditions**: A raw PDF path operator sequence (e.g., `m 0 0 l 10 10 h 20 v 20 z`).
- **Test Steps**:
    1. Call `SvgRenderer::convert_path(operators)`.
    2. Compare output against expected SVG `d` attribute.
- **Expected Result**: Output string equals `"M 0 0 L 10 10 H 30 V 30 Z"`.
- **Actual Files**: `src/engine/renderer/svg.rs`

### TC-004: Absolute Positioning Calculation
- **Description**: Verify the coordinate transformation from PDF points to HTML pixel-based absolute positioning.
- **Preconditions**: A PDF element at coordinates (72, 72) on an 8.5x11 inch page.
- **Test Steps**:
    1. Call `LayoutEngine::calculate_bounds(element, scale_factor: 1.0)`.
- **Expected Result**: Returns a CSS rect with `top: 96px`, `left: 96px` (assuming 96 DPI).
- **Actual Files**: `src/engine/layout.rs`

### TC-005: Wasm Buffer Serialization (Shared Memory)
- **Description**: Ensure large PDF buffers can be passed from TypeScript to Rust via Wasm memory without corruption.
- **Preconditions**: A 10MB `Uint8Array` in the browser environment.
- **Test Steps**:
    1. Call `wasm_bridge.alloc(size)`.
    2. Write buffer to Wasm memory.
    3. Call `process_pdf(ptr, size)` in Rust.
- **Expected Result**: Rust receives the exact byte sequence; no memory leaks reported by `wee_alloc`.
- **Actual Files**: `src/wasm/lib.rs`, `src/wasm/memory.rs`

### TC-006: Job Metadata Database Persistence Logic
- **Description**: Validate the logic for preparing PostgreSQL insertion queries for new conversion jobs.
- **Preconditions**: A `JobRequest` struct with valid UUIDs and timestamps.
- **Test Steps**:
    1. Call `JobRepository::create_query(job_request)`.
    2. Verify the SQL string and parameter binding order.
- **Expected Result**: SQL matches the schema; all fields (status, input_path, created_at) are mapped.
- **Actual Files**: `src/api/db/jobs.rs`

### TC-007: Redis Cache Key Generation
- **Description**: Ensure cache keys are deterministic based on PDF hash and conversion settings.
- **Preconditions**: Two identical PDF files with different filenames.
- **Test Steps**:
    1. Generate hash for File A and File B.
    2. Call `CacheManager::get_key(hash, settings)`.
- **Expected Result**: Both files generate the identical key `pdf:cache:<hash>:<settings_id>`.
- **Actual Files**: `src/api/cache/redis.rs`

### TC-008: Property-Based Path Sanitization (Proptest)
- **Description**: Use property-based testing to ensure file path sanitization prevents directory traversal.
- **Preconditions**: `proptest` crate configured.
- **Test Steps**:
    1. Generate random strings using `any::<String>()`.
    2. Call `PathUtils::sanitize(input)`.
    3. Assert that the output never contains `..` or starts with `/`.
- **Expected Result**: 1000+ random iterations pass without returning an unsafe path.
- **Actual Files**: `src/utils/path_sanitizer.rs`

### TC-009: React Viewer Zoom State Logic
- **Description**: Test the state reducer for the document viewer zoom functionality.
- **Preconditions**: Initial zoom state at 1.0.
- **Test Steps**:
    1. Dispatch `ZOOM_IN` action.
    2. Dispatch `SET_ZOOM` with value 5.0 (exceeding max 3.0).
- **Expected Result**: Zoom increments to 1.1; `SET_ZOOM` caps the value at 3.0.
- **Actual Files**: `ui/components/viewer/useZoom.ts`

### TC-010: Corrupted PDF Error Handling
- **Description**: Verify the engine returns a specific `OdeError::InvalidFormat` when given a non-PDF file.
- **Preconditions**: A text file renamed to `.pdf`.
- **Test Steps**:
    1. Call `PdfDocument::new(corrupted_buffer)`.
- **Expected Result**: Returns `Err(OdeError::InvalidFormat)`; does not panic.
- **Actual Files**: `src/engine/error.rs`, `src/engine/pdf_parser.rs`

---

## 3. Test Data

### Sample Data
- `minimal.pdf`: A single-page PDF with one text string ("Hello World").
- `complex_vectors.pdf`: A PDF containing overlapping SVG paths and gradients.
- `multi_font.pdf`: A document using Type1, TrueType, and OpenType fonts.

### Edge Cases
- **Zero-byte files**: Handling empty input buffers.
- **Password-protected PDFs**: Ensuring the engine returns an `Unauthorized` error rather than crashing.
- **Extremely large coordinates**: PDF elements placed far outside the media box.
- **Malformed UTF-8**: PDF metadata containing invalid character sequences.

### Error Scenarios
- **Redis Connection Timeout**: Logic for bypassing cache when Redis is unreachable.
- **Memory Exhaustion**: Graceful failure when the Wasm heap limit is reached.

---

## 4. Verification Checklist

- [ ] **TC-001 to TC-010**: All test cases have explicit `assert_eq!` or `assert_matches!` statements.
- [ ] **Code Coverage**: Target > 85% coverage for `ode-core`.
- [ ] **Memory Safety**: Run unit tests with `cargo miri test` to detect undefined behavior.
- [ ] **Property Testing**: `proptest` covers at least 3 critical modules (Parser, Sanitizer, Layout).
- [ ] **Error Propagation**: All `Result` types are checked; no `unwrap()` or `expect()` in production paths.

---

## 5. Implementation Specification

### Acceptance Criteria
- [ ] All unit tests pass in the GitHub Actions CI pipeline.
- [ ] No memory leaks detected during `ode-wasm` unit tests.
- [ ] `cargo test` execution time remains under 60 seconds for the entire suite.

### File References
- **New Tests**: `src/tests/unit_engine_tests.rs`, `src/tests/unit_api_tests.rs`.
- **Modified**: `src/lib.rs` (to include test modules).

### Dependencies
1. **Pre-requisite**: Rust toolchain (latest stable), `wasm-pack`.
2. **Post-requisite**: Integration tests (API level) and Visual Regression tests (Playwright).

### Verification Steps
1. Run `cargo test` locally to verify the core logic.
2. Run `wasm-pack test --node` to verify the Wasm bridge logic.
3. Execute `cargo tarpaulin --ignore-tests` to generate a coverage report.
4. Inspect `target/tarpaulin/report.html` to ensure all critical paths in `pdf_parser.rs` are covered.