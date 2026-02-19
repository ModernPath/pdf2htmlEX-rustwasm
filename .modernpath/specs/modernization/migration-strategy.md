# Modernization Specification: Migration Strategy
**Project**: pdf2htmlEX to Oxidized Document Engine (ODE) / rust-pdf2html  
**Status**: Draft / For Review  
**Expert**: Senior Software Modernization Expert  

---

## 1. Executive Summary
The goal is to modernize the legacy C/C++ `pdf2htmlEX` utility into a high-performance, memory-safe Rust-based engine (ODE). The migration strategy follows a **Parallel Run & Functional Parity** approach. Given the complexity of PDF rendering, we will not perform a line-by-line port but rather a functional rewrite that maps the four-layer architecture of the source to a modern asynchronous microservice and WebAssembly (Wasm) client.

---

## 2. Source Analysis

### Key Business Logic
*   **Graphics State Tracking**: Maintaining transformations, clipping paths, and color spaces during PDF traversal.
*   **Font Extraction**: Utilizing FontForge wrappers to convert embedded PDF fonts into web-ready formats (WOFF/TTF).
*   **Dual-Engine Rendering**: Orchestrating the `HTMLRenderer` (for semantic text) and `BackgroundRenderer` (for SVG/image visual fidelity).

### Dependencies
*   **Core**: Poppler (PDF parsing), FontForge (Font processing).
*   **Rendering**: Cairo (SVG), Splash (Bitmaps).
*   **Platform**: MinGW/Windows compatibility layers in `util`.

### Data Structures
*   **In-Memory**: C++ structures for PDF page trees, font maps, and graphics state stacks.
*   **Output**: Static HTML/CSS files with Base64 encoded assets.

### Integration Points
*   **CLI**: Primary entry point via command-line arguments.
*   **Filesystem**: Direct read/write of PDF and HTML artifacts.

---

## 3. Target Architecture Mapping

| Source Pattern (C/C++) | Target Pattern (Rust/ODE) | Technology Change |
| :--- | :--- | :--- |
| **Orchestration Layer** | **Axum API + Tokio** | CLI $\rightarrow$ Scalable Async Service |
| **HTML Rendering Engine** | **Rust Core Engine** | C++ $\rightarrow$ Memory-safe Rust (`pdf-rs`/`lopdf`) |
| **Background Rendering** | **Wasm + SVG/Canvas** | Server-side Cairo $\rightarrow$ Client-side Wasm/Browser |
| **Shared Utilities** | **Rust Crates (Workspace)** | Manual C utils $\rightarrow$ Standardized Crates |
| **In-memory State** | **PostgreSQL + Redis** | Transient State $\rightarrow$ Persistent Job Metadata |

---

## 4. Phasing Strategy

### Phase 1: The "Oxidizer" (Core Engine)
*   **Goal**: Achieve 1:1 text extraction and graphics state parity in Rust.
*   **Focus**: Porting `HTMLRenderer` logic.
*   **Source**: `src/HTMLRenderer/` $\rightarrow$ **Target**: `crates/ode-core/`

### Phase 2: The "Broker" (API & Orchestration)
*   **Goal**: Wrap the core engine in a production-ready API.
*   **Focus**: Replacing `main.cpp` logic with Axum routes and Redis task queues.
*   **Source**: `src/main.cpp` $\rightarrow$ **Target**: `services/api-gateway/`

### Phase 3: The "Viewer" (Wasm & UI)
*   **Goal**: Offload background rendering to the client browser.
*   **Focus**: Compiling core components to Wasm for React integration.
*   **Source**: `src/BackgroundRenderer/` $\rightarrow$ **Target**: `packages/ode-viewer-wasm/`

### Phase 4: Productionization & Cutover
*   **Goal**: Infrastructure as Code (Terraform) and EKS deployment.
*   **Focus**: Monitoring (Prometheus/Grafana) and final traffic shift.

---

## 5. Transformation Steps (Component: Core Engine)

- [ ] **Step 1: PDF Parsing Foundation**  
    Implement PDF tree traversal using the `lopdf` or `pdf` crate to replace Poppler-based ingestion.  
    *Source: `src/main.cpp` (Preprocessing)* $\rightarrow$ *Target: `ode-core/src/parser.rs`*
- [ ] **Step 2: Graphics State Implementation**  
    Create a Rust `struct` to track `GraphicsState` (CTM, Clipping, Colors) mirroring the logic in `HTMLRenderer`.  
    *Source: `src/HTMLRenderer/HTMLRenderer.cpp`* $\rightarrow$ *Target: `ode-core/src/render/state.rs`*
- [ ] **Step 3: Font Processing Bridge**  
    Develop a Rust wrapper for FontForge or integrate a native Rust font-subsetting library (`face` or `rusttype`).  
    *Source: `src/util/font.cpp`* $\rightarrow$ *Target: `ode-core/src/fonts/`*

---

## 6. Risk Assessment

| Risk Area | Complexity | Mitigation Strategy |
| :--- | :--- | :--- |
| **Font Fidelity** | High | Use `proptest` to compare glyph bounding boxes between C++ and Rust outputs. |
| **Memory Safety** | Medium | Leverage Rust's ownership model; strictly audit any `unsafe` blocks used for C-interop. |
| **Performance** | Medium | Implement Redis caching for frequently processed document segments. |
| **Visual Regression** | High | Use **Playwright Visual Regression** to compare pixel-diffs of HTML output. |

### Rollback Plan
1.  **Shadow Mode**: Run ODE in parallel with `pdf2htmlEX` in production. Compare outputs but only serve `pdf2htmlEX` results to users.
2.  **Feature Flag**: Use a LaunchDarkly or custom Redis-based flag to toggle the engine per tenant or document type.
3.  **Legacy Proxy**: If ODE fails, the Axum API will automatically fallback to a legacy Docker container running the original C++ CLI.

---

## 7. Verification Checklist & Test Strategy

### Verification Steps
- [ ] **Functional Parity**: Run 1,000 sample PDFs through both engines; HTML DOM structure must match >98%.
- [ ] **Memory Audit**: `cargo-valgrind` execution to ensure zero leaks in the new Rust core.
- [ ] **Wasm Performance**: Ensure document load time in React is < 2s for a 50-page PDF.

### Test Cases
| ID | Scenario | Expected Outcome |
| :--- | :--- | :--- |
| **TC-01** | Complex Vector Path (SVG) | ODE output matches legacy pixel-for-pixel (threshold 0.1%). |
| **TC-02** | Multi-byte Unicode Fonts | Text remains searchable and selectable in the generated HTML. |
| **TC-03** | Concurrent Job Spikes | Axum/Tokio handles 500 concurrent conversion requests without OOM. |

---

## 8. Acceptance Criteria
1.  **Memory Safety**: 100% of the core engine is written in safe Rust, or `unsafe` blocks are documented and audited.
2.  **Visual Fidelity**: Automated visual diff tests show no regressions on the "Standard PDF Test Suite."
3.  **API Latency**: P99 latency for metadata extraction is < 200ms.
4.  **Accessibility**: Generated HTML passes WCAG 2.1 AA audits using Axe-core.

---
**Dependencies**: 
*   Completion of `ode-core` (Phase 1) is required before `api-gateway` (Phase 2) integration.
*   Terraform modules for EKS must be ready before Phase 4.