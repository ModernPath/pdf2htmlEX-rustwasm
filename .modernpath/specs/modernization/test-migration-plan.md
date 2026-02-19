# Modernization Specification: Test Migration Plan
**Project:** Oxidized Document Engine (ODE) / rust-pdf2html  
**Target Architecture:** Rust-based Document Transformation Engine  
**Source System:** pdf2htmlEX (C/C++)

---

## 1. Executive Summary
The goal of this specification is to define a comprehensive testing strategy for the modernization of `pdf2htmlEX` into the **Oxidized Document Engine (ODE)**. Because the core value proposition of the system is "pixel-perfect rendering" and "visual fidelity," the testing strategy shifts from traditional unit testing to a heavy emphasis on **Visual Regression Testing (VRT)** and **Property-Based Testing** to ensure the Rust-based engine matches or exceeds the legacy C++ output.

---

## 2. Source Analysis (Legacy pdf2htmlEX)

### Key Business Logic
*   **Transformation Pipeline**: Ingests binary PDF data, maintains a complex "Graphics State" (transformations, clipping paths), and serializes it into HTML/CSS.
*   **Font Processing**: Uses FontForge C-wrappers to extract and convert embedded fonts.
*   **Layout Preservation**: Calculates absolute positioning for text fragments to ensure they overlap exactly with the PDF source.

### Dependencies
*   **Core**: Cairo (SVG), Splash (Bitmaps), FontForge (Fonts), Poppler/PDF.js (Parsing).
*   **Testing Infrastructure**: Primarily CLI-based. Tests likely involve running the binary against a directory of PDFs and checking for exit codes or performing manual diffs on generated HTML files.

### Data Structures
*   **In-Memory Graphics State**: C++ structures representing the current color, font, and transformation matrix.
*   **Text Streams**: Sequential buffers of character data with associated coordinate metadata.

### Integration Points
*   **CLI**: Standard input/output for file paths and configuration flags.
*   **Filesystem**: Heavy reliance on temporary files for font extraction and image rendering.

---

## 3. Target Architecture Mapping

| Legacy Pattern (C++) | Target Pattern (Rust/ODE) | Technology Change |
| :--- | :--- | :--- |
| CLI-based manual diffs | Automated Visual Regression | Playwright + Pixelmatch |
| Ad-hoc error codes | Result-based Error Handling | Rust `Result<T, E>` + `anyhow` |
| Manual edge-case testing | Property-based Testing | `proptest` crate |
| Single-threaded processing | Async Pipeline Testing | `tokio::test` |
| Browser-based manual check | Headless VRT | Playwright (Chromium/Webkit) |

---

## 4. Transformation Steps

### Phase 1: Baseline Generation (The "Oracle")
*   [ ] **Step 1**: Create a "Golden Dataset" of 100+ PDFs covering various complexities (ligatures, transparent layers, embedded Type 3 fonts).
*   [ ] **Step 2**: Execute the legacy `pdf2htmlEX` against this dataset.
*   [ ] **Step 3**: Capture and store the output HTML and high-resolution screenshots (via headless Chrome) as the "Ground Truth."
*   **Source Reference**: `src/main.cc` (CLI Entry) → **Target**: `tests/baselines/`

### Phase 2: Unit & Logic Migration
*   [ ] **Step 1**: Port geometric utility functions (matrix transformations, bounding box intersections).
*   [ ] **Step 2**: Implement Rust unit tests for these utilities using `cargo test`.
*   **Source Reference**: `src/util/` → **Target**: `crates/ode-core/src/util.rs`

### Phase 3: Visual Regression Integration
*   [ ] **Step 1**: Set up a Playwright test suite that renders the output of the new Rust engine.
*   [ ] **Step 2**: Implement a comparison logic that compares the Rust-generated HTML screenshot against the C++ "Ground Truth."
*   **Target**: `tests/visual_regression.spec.ts`

### Phase 4: Property-Based Testing (Fuzzing)
*   [ ] **Step 1**: Use `proptest` to generate random valid/invalid PDF-like structures to ensure the Rust engine handles memory safety and crashes gracefully.
*   **Target**: `crates/ode-core/tests/property_tests.rs`

---

## 5. Risk Assessment

| Risk Area | Severity | Mitigation Strategy |
| :--- | :--- | :--- |
| **Font Rendering Diffs** | High | Minor anti-aliasing differences between Cairo (C++) and the new Rust font stack are expected. Use a %-based threshold for visual diffs. |
| **Wasm Compatibility** | Medium | Ensure tests run both in native Rust and via `wasm-bindgen-test` in a browser environment. |
| **Performance Regression** | Medium | Integrate `criterion.rs` for micro-benchmarking critical rendering paths. |
| **Data Loss (Metadata)** | Low | Verify PostgreSQL job metadata integrity via transaction-rollback tests in Axum. |

---

## 6. Test Strategy & Verification

### Acceptance Criteria
- [ ] **Visual Fidelity**: 99.5% pixel similarity across the Golden Dataset compared to legacy output.
- [ ] **Memory Safety**: Zero `unsafe` blocks in the rendering engine unless audited and documented.
- [ ] **Accessibility**: Generated HTML must pass `axe-core` accessibility audits (WCAG 2.1 AA).
- [ ] **Performance**: Rust engine must process documents ≥ 20% faster than the C++ legacy version.

### Specific Test Cases

#### Test Case 1: Complex Layout Preservation
*   **Input**: A 3-column scientific paper PDF with mathematical formulas.
*   **Action**: Convert via ODE.
*   **Expected Behavior**: Text fragments in HTML must be within 1px of the original PDF position.
*   **Verification**: Playwright screenshot comparison with a 0.1% failure threshold.

#### Test Case 2: Font Encoding (Unicode)
*   **Input**: PDF containing non-standard glyphs (CJK, RTL languages).
*   **Action**: Extract fonts and generate HTML.
*   **Expected Behavior**: Text must be searchable and copy-pasteable as valid Unicode.
*   **Verification**: Automated check of `innerText` against a known Unicode string.

#### Test Case 3: Job Queue Resilience
*   **Input**: 50 concurrent conversion requests to the Axum API.
*   **Action**: Simulate a Redis connection failure mid-process.
*   **Expected Behavior**: Jobs are marked as "failed" in PostgreSQL and can be retried without data corruption.
*   **Verification**: Integration test using `testcontainers-rs` to kill the Redis container.

### File References
*   **New Test Suite**: `tests/integration_test.rs` (Full pipeline)
*   **Visual Config**: `playwright.config.ts`
*   **Benchmarking**: `benches/rendering_bench.rs`
*   **Mock Data**: `tests/fixtures/*.pdf`

---

## 7. Rollback & Verification Steps
1.  **Verification**: Run `cargo test` and `npm run test:visual`. All must pass.
2.  **Staging Validation**: Deploy to the Staging environment; run the Playwright suite against the EKS endpoint.
3.  **Rollback Plan**: If visual fidelity falls below 95% on production traffic, revert the K8s deployment to the previous image tag and trigger a "Test Failure" post-mortem.

---
**Dependencies**: 
- Must complete `ode-core` (Rust rendering engine) before Phase 3.
- Must complete `infrastructure-as-code` (Terraform) before Phase 4.