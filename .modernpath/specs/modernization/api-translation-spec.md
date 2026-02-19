# API Translation Specification: pdf2htmlEX to Oxidized Document Engine (ODE)

This document specifies the transformation of the `pdf2htmlEX` command-line interface and internal C++ logic into a modern, asynchronous REST API and WebAssembly (Wasm) interface.

---

## 1. Source Analysis

### Key Business Logic
The source system (`pdf2htmlEX`) is a **synchronous pipeline** that transforms binary PDF data into a complex DOM structure.
- **Orchestration**: Handled in `main.cc`, managing the lifecycle of a conversion.
- **State Management**: The `HTMLRenderer` maintains a "Graphics State" (colors, transforms, clipping) across page boundaries.
- **Asset Generation**: Font extraction (via FontForge) and background image rendering (via Cairo/Splash).

### Dependencies
- **Cairo/Splash**: Vector and raster rendering.
- **FontForge**: C-wrapper for font conversion.
- **Poppler (implied)**: Underlying PDF parsing logic.

### Data Structures
- **Internal**: C++ structs for `GfxState`, `TextLine`, and `BBox`.
- **Output**: Static files (HTML, CSS, WOFF, PNG/SVG).

### Integration Points
- **CLI**: The primary interface for users.
- **Filesystem**: Heavy reliance on local I/O for temporary and final artifacts.

---

## 2. Target Architecture Mapping

| Feature | Source Pattern (C++/CLI) | Target Pattern (Rust/Axum/ODE) |
| :--- | :--- | :--- |
| **Interface** | CLI Arguments / Stdout | REST API (JSON) / Wasm Exports |
| **Concurrency** | Single-process / Manual Forking | Tokio Async Tasks / Worker Pool |
| **State** | In-memory C++ Objects | Redis-backed Job State / Postgres Metadata |
| **Storage** | Local Filesystem | S3-compatible Object Storage |
| **Rendering** | Cairo (C-based) | `pdf-rs` or `resvg` (Rust-native) |
| **Font Logic** | FontForge C-Wrapper | `rusttype` / `font-kit` / Wasm-compatible libs |

---

## 3. API Translation Matrix

The CLI flags from `pdf2htmlEX` will be mapped to a JSON request body for the ODE `POST /v1/jobs` endpoint.

### Request Mapping (CLI to JSON)
| CLI Flag | JSON Property (ODE API) | Type | Description |
| :--- | :--- | :--- | :--- |
| `input.pdf` | `document_url` or Multipart | String/File | Source PDF document |
| `--first-page` | `page_range.start` | Integer | First page to convert |
| `--last-page` | `page_range.end` | Integer | Last page to convert |
| `--zoom` | `render_options.zoom_factor` | Float | Scaling factor |
| `--embed-font` | `output_options.embed_fonts` | Boolean | Whether to Base64 fonts in CSS |
| `--data-dir` | *N/A (Environment Var)* | String | Path to shared assets |

### New API Endpoints (Axum)
- `POST /v1/convert`: Immediate conversion (for small files/Wasm).
- `POST /v1/jobs`: Asynchronous conversion (returns `job_id`).
- `GET /v1/jobs/{id}`: Status polling and result URLs.
- `GET /v1/jobs/{id}/download`: Zip archive of all assets.

---

## 4. Transformation Steps

### Component: Orchestration & API Layer
- [ ] **Step 1**: Implement Axum routes and JSON serialization using `Serde`.
- [ ] **Step 2**: Integrate `Tokio` for non-blocking execution of the conversion core.
- [ ] **Step 3**: Implement a Redis-based task queue to handle high-throughput batch processing.
- **Source**: `src/main.cc` → **Target**: `src/api/routes.rs`, `src/api/handlers.rs`

### Component: Rendering Engine (The "Oxidized" Core)
- [ ] **Step 1**: Port `HTMLRenderer` logic to Rust, focusing on the `GfxState` manager.
- [ ] **Step 2**: Replace Cairo calls with a Rust-native vector crate (e.g., `vello` or `resvg`).
- [ ] **Step 3**: Compile the core engine to `wasm32-unknown-unknown` for client-side browser execution.
- **Source**: `src/HTMLRenderer.cc`, `src/BackgroundRenderer.cc` → **Target**: `crate::core::renderer`, `crate::core::wasm_interface`

### Component: Asset Management
- [ ] **Step 1**: Create an abstraction layer for storage (Local vs. S3).
- [ ] **Step 2**: Implement font conversion logic using Rust-native font tools to remove FontForge dependency.
- **Source**: `src/util/path.cc` → **Target**: `crate::infra::storage`, `crate::core::fonts`

---

## 5. Risk Assessment

| Risk | Impact | Mitigation |
| :--- | :--- | :--- |
| **Visual Regression** | High | Use Playwright for pixel-perfect comparison between `pdf2htmlEX` and ODE output. |
| **Memory Exhaustion** | Medium | Implement Axum middleware for request size limits and Tokio resource quotas. |
| **Font Licensing** | Low | Ensure Rust font libraries handle font embedding flags (fsType) correctly. |
| **Wasm Performance** | Medium | Use `wasm-bindgen` with `SharedArrayBuffer` for multi-threaded Wasm rendering. |

---

## 6. Verification Checklist & Test Strategy

### Acceptance Criteria
- [ ] **API Functional**: `POST /v1/jobs` returns 202 Accepted with a valid UUID.
- [ ] **Parity**: HTML output from ODE matches `pdf2htmlEX` visual layout within a 0.5% pixel delta.
- [ ] **Security**: No raw pointers or `unsafe` blocks in the API parsing layer.
- [ ] **Scalability**: System handles 50 concurrent PDF conversions without crashing.

### Test Cases
1. **Scenario**: Convert a 100-page PDF with embedded Type 1 fonts.
   - **Expected**: API returns a "completed" status; fonts are converted to WOFF2 and visible in the browser.
2. **Scenario**: Wasm execution in Chrome.
   - **Expected**: A `Uint8Array` of the PDF is converted to a DOM string locally without server round-trips.
3. **Scenario**: Invalid PDF upload.
   - **Expected**: API returns `422 Unprocessable Entity` with a JSON error describing the parsing failure.

### Verification Steps
1. [ ] Run `cargo test` to verify unit logic in the `GfxState` manager.
2. [ ] Execute the `visual-diff` suite:
   - Run `pdf2htmlEX sample.pdf` -> `old.html`
   - Run `curl -X POST ... ODE/v1/convert` -> `new.html`
   - Run `playwright-compare old.html new.html`
3. [ ] Verify OpenTelemetry traces in Grafana for a single job lifecycle.

---

## 7. File Mapping Reference

| Source File (C++) | Target File (Rust/TS) | Responsibility |
| :--- | :--- | :--- |
| `src/HTMLRenderer.cc` | `src/core/render/html.rs` | DOM generation and text placement |
| `src/BackgroundRenderer.cc` | `src/core/render/canvas.rs` | SVG/Image background generation |
| `src/main.cc` | `src/main.rs` | CLI Entrypoint (Legacy Support) |
| `N/A` | `src/api/mod.rs` | **New** Axum REST API layer |
| `src/util/unicode.cc` | `src/core/text/unicode.rs` | Unicode mapping and normalization |