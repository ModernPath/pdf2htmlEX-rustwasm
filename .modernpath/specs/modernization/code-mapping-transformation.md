# Modernization Specification: Code Mapping & Transformation
**Project**: Oxidized Document Engine (ODE)  
**Source System**: pdf2htmlEX (C/C++)  
**Target Architecture**: Rust-based Wasm/Axum Microservice  

---

## 1. Source Analysis

### Key Business Logic
The core value of `pdf2htmlEX` lies in its **Transformation Pipeline**, which converts binary PDF streams into a combination of semantic HTML (for text) and background layers (SVG/Images for graphics). 
- **Graphics State Tracking**: Maintaining the Current Transformation Matrix (CTM), clipping paths, and color spaces.
- **Font Processing**: Extracting embedded fonts and converting them to web-compatible formats (WOFF/TTF) while maintaining character mapping (ToUnicode).
- **Occlusion Detection**: Determining if text is hidden behind images to optimize the DOM.

### Dependencies & Integration Points
- **Source Dependencies**: `Poppler` (PDF parsing), `FontForge` (Font conversion), `Cairo` (SVG generation), `Splash` (Rasterization).
- **Integration**: Currently a CLI tool. Integration is file-system based (Input PDF $\rightarrow$ Output Directory).

### Data Structures
- **Internal**: C++ structures representing the `GfxState`, `Page`, and `Font` objects from the Poppler library.
- **Output**: A complex manifest of HTML, CSS, and font files, often bundled into a single "fat" HTML using Base64.

---

## 2. Target Architecture Mapping

| Source Pattern (C++/CLI) | Target Pattern (Rust/Axum/Wasm) | Rationale |
| :--- | :--- | :--- |
| **Monolithic CLI Orchestration** | **Axum API + Tokio Workers** | Enables horizontal scaling and async job processing. |
| **Poppler / GfxState** | **`pdf-rs` or `lopdf` + Custom State Machine** | Memory-safe PDF parsing without C-bindings where possible. |
| **FontForge (C-Wrapper)** | **`swash` / `ttf-parser` / `rusttype`** | Replaces heavy C-dependencies with native Rust font-handling. |
| **Cairo/Splash Rendering** | **`resvg` / `tiny-skia`** | High-performance, memory-safe vector and raster rendering. |
| **In-memory C++ Structs** | **Serde-compatible Rust Structs** | Simplifies metadata storage in PostgreSQL and caching in Redis. |
| **Manual Memory Mgmt** | **RAII / Ownership & Borrowing** | Eliminates memory leaks and segmentation faults common in PDF parsing. |

---

## 3. Transformation Steps

### Phase 1: Core Engine (The "Oxidizer")
Focus on the low-level translation of PDF operators to Rust structures.

- [ ] **Step 1: PDF Stream Ingestion**
    - **Action**: Implement a PDF parser using the `lopdf` crate to extract page objects and resource dictionaries.
    - **Source Reference**: `src/pdf2htmlEX.cc` (Main loop)
    - **Target**: `ode-core/src/parser/mod.rs`
- [ ] **Step 2: Graphics State Tracker**
    - **Action**: Port the C++ `GfxState` logic to a Rust `struct` that tracks transformations, clipping, and transparency.
    - **Source Reference**: `src/HTMLRenderer/HTMLRenderer.cc`
    - **Target**: `ode-core/src/render/state.rs`

### Phase 2: Asset Transformation
- [ ] **Step 3: Font Extraction Service**
    - **Action**: Replace FontForge calls with a Rust-native pipeline. Use `ttf-parser` to read glyph data and `woff2` crate for compression.
    - **Source Reference**: `src/FontDescriptor.cc`, `src/util/font.cc`
    - **Target**: `ode-core/src/fonts/processor.rs`
- [ ] **Step 4: Background Layer Generation**
    - **Action**: Use `tiny-skia` to render non-text PDF elements into SVG or WebP.
    - **Source Reference**: `src/BackgroundRenderer/CairoBackgroundRenderer.cc`
    - **Target**: `ode-core/src/render/background.rs`

### Phase 3: API & Distribution
- [ ] **Step 5: Wasm Wrapper**
    - **Action**: Expose the core engine via `wasm-bindgen` for client-side rendering in the React component.
    - **Target**: `ode-wasm/src/lib.rs`
- [ ] **Step 6: Axum Job Wrapper**
    - **Action**: Create the REST entry point that accepts a PDF, queues a job in Redis, and returns a JobID.
    - **Target**: `ode-api/src/routes/jobs.rs`

---

## 4. Risk Assessment

| Risk | Severity | Mitigation |
| :--- | :--- | :--- |
| **Font Fidelity Loss** | High | Use `proptest` to compare glyph bounding boxes between FontForge and new Rust implementation. |
| **Memory Exhaustion** | Medium | Implement Tokio-based resource limits and stream PDF objects instead of loading the full document. |
| **Wasm Performance** | Medium | Use `web-sys` for hardware-accelerated canvas rendering in the browser. |
| **Complex Clipping Paths** | High | Extensive visual regression testing using Playwright against the original `pdf2htmlEX` output. |

---

## 5. Verification Checklist & Test Strategy

### Test Strategy
1. **Differential Testing**: Run 1,000 sample PDFs through both `pdf2htmlEX` and `ODE`. 
2. **Visual Regression**: Use Playwright to take screenshots of both outputs and use pixel-diffing (threshold < 0.1%).
3. **Property-Based Testing**: Use `proptest` in Rust to ensure the `GfxState` matrix math never produces `NaN` or `Inf`.

### Acceptance Criteria
- [ ] **Visual Parity**: 99% pixel match on text positioning compared to source system.
- [ ] **Memory Safety**: Zero `unsafe` blocks in the `ode-core` crate (unless wrapping a specific sys-library).
- [ ] **Performance**: Conversion speed must be $\le$ 1.2x of the C++ implementation.
- [ ] **Accessibility**: Generated HTML must pass `axe-core` accessibility audits (WCAG 2.1 AA).

### Verification Steps
1. [ ] **Unit Test**: `cargo test` for all coordinate transformation functions.
2. [ ] **Integration Test**: Submit a PDF to the Axum API and verify the Redis task status moves to `COMPLETED`.
3. [ ] **Wasm Test**: Load a 10MB PDF in the React sandbox and verify it renders in < 2 seconds.
4. [ ] **Visual Diff**: Run `playwright test` comparing `legacy-output/*.html` vs `ode-output/*.html`.

---

## 6. File Mapping Reference

| Source File (Legacy) | Target File (Modern) | Status |
| :--- | :--- | :--- |
| `src/HTMLRenderer/HTMLRenderer.cc` | `ode-core/src/render/html_writer.rs` | To Be Created |
| `src/BackgroundRenderer/Cairo.cc` | `ode-core/src/render/svg_backend.rs` | To Be Created |
| `src/util/unicode.cc` | `ode-utils/src/unicode.rs` | To Be Created |
| `pdf2htmlEX.cc` (main) | `ode-api/src/main.rs` | To Be Created |
| `share/manifest` | `ode-core/src/manifest.rs` | To Be Created |