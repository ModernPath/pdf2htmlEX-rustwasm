This document outlines the **Core Features Epic** for the **Oxidized Document Engine (ODE)**. These user stories focus on the fundamental transformation engine, the asynchronous processing pipeline, and the high-fidelity rendering components.

---

## User Stories: Epic - Core Features

### US-001: High-Performance Rust-Based PDF to HTML Conversion
**As a** Backend Developer  
**I want to** utilize a Rust-based engine to convert PDF documents into HTML/CSS  
**So that** I can achieve high-speed, memory-safe transformations that preserve the original document's visual fidelity.

*   **Acceptance Criteria**:
    *   [ ] The engine must successfully parse PDF 1.7+ specifications using Rust.
    *   [ ] Output HTML must utilize absolute positioning and CSS transforms to match PDF coordinates within a 0.5pt margin of error.
    *   [ ] Text elements must be extracted as semantic HTML tags (e.g., `<span>` or `<div>`) rather than flat images.
    *   [ ] Memory usage during conversion must not exceed 2x the size of the source PDF file for documents under 50MB.
*   **Test Scenarios**:
    *   **Scenario 1**: Convert a 10-page complex layout PDF. **Result**: Visual regression test (Playwright) shows >99% similarity to original PDF.
    *   **Scenario 2**: Process a corrupted PDF file. **Result**: Engine returns a graceful `Result::Err` with a specific error code instead of panicking.
*   **Affected Components**: `ode-core` (Rust crate), `pdf-parser` module.
*   **Story Estimation**: 
    *   Complexity: High
    *   Dependencies: None (Base Engine)
*   **File References**: `src/engine/converter.rs`, `src/engine/layout_engine.rs`
*   **Verification Steps**: Run `cargo test --package ode-core` and verify visual output via `playwright test --project=visual-regression`.

---

### US-002: Asynchronous Job Submission API
**As a** Frontend Engineer  
**I want to** submit a PDF to a REST API and receive a Job ID  
**So that** I can process large documents in the background without blocking the UI.

*   **Acceptance Criteria**:
    *   [ ] POST `/api/v1/convert` endpoint accepts `multipart/form-data` (PDF file + configuration).
    *   [ ] API returns a `202 Accepted` status with a unique UUID Job ID.
    *   [ ] Job metadata (filename, size, timestamp) is persisted in PostgreSQL.
    *   [ ] The task is successfully pushed to a Redis-backed queue for processing.
*   **Test Scenarios**:
    *   **Scenario 1**: Submit a valid PDF. **Result**: Receive UUID and verify entry in `jobs` table.
    *   **Scenario 2**: Submit a non-PDF file. **Result**: Receive `415 Unsupported Media Type`.
*   **Affected Components**: `ode-api` (Axum), `ode-worker` (Tokio), PostgreSQL, Redis.
*   **Story Estimation**: 
    *   Complexity: Medium
    *   Dependencies: US-001
*   **File References**: `src/api/routes/convert.rs`, `src/db/models/job.rs`
*   **Verification Steps**: Use `curl` to POST a file and verify the response schema matches OpenAPI specs. Check Redis via `redis-cli LLEN job_queue`.

---

### US-003: Client-Side Wasm Conversion Module
**As a** Frontend Developer  
**I want to** run the conversion engine directly in the browser via WebAssembly  
**So that** I can reduce server load and provide instant previews for small documents.

*   **Acceptance Criteria**:
    *   [ ] Compile the Rust core to `wasm32-unknown-unknown` target.
    *   [ ] Provide a TypeScript wrapper to initialize the Wasm module and call `convert_to_html()`.
    *   [ ] Wasm execution must be performed in a Web Worker to prevent UI thread blocking.
    *   [ ] The module size (gzipped) must be under 2MB.
*   **Test Scenarios**:
    *   **Scenario 1**: Load Wasm in Chrome/Firefox and convert a 1MB PDF. **Result**: HTML generated in <2 seconds.
    *   **Scenario 2**: Run conversion on a low-memory mobile browser. **Result**: Module handles memory limits without crashing the tab.
*   **Affected Components**: `ode-wasm` (Rust/Wasm-pack), `ode-client-sdk` (TS).
*   **Story Estimation**: 
    *   Complexity: High
    *   Dependencies: US-001
*   **File References**: `wasm/src/lib.rs`, `packages/sdk/src/wasm-worker.ts`
*   **Verification Steps**: Run `wasm-pack test --headless --chrome`.

---

### US-004: React Document Viewer Component
**As a** Frontend Engineer  
**I want to** use a pre-built React component to display the converted HTML  
**So that** I can easily integrate document viewing into my application with built-in zoom and navigation.

*   **Acceptance Criteria**:
    *   [ ] Component accepts a `jobId` or `rawHtml` as a prop.
    *   [ ] Implements Radix UI primitives for accessible controls (Zoom, Page Navigation).
    *   [ ] Supports "Lazy Loading" of pages to handle 100+ page documents efficiently.
    *   [ ] Complies with WCAG 2.1 AA (screen reader support for extracted text).
*   **Test Scenarios**:
    *   **Scenario 1**: Pass a 50-page HTML string to the viewer. **Result**: Only the first 3 pages render initially; others render on scroll.
    *   **Scenario 2**: Tab through the viewer. **Result**: Focus indicators are visible and follow a logical order.
*   **Affected Components**: `ode-ui-react` (React, Tailwind, Radix).
*   **Story Estimation**: 
    *   Complexity: Medium
    *   Dependencies: US-002
*   **File References**: `ui/src/components/DocumentViewer.tsx`, `ui/src/hooks/useDocumentLoader.ts`
*   **Verification Steps**: Run `npm run test:ui` (Vitest) and manual audit using Axe DevTools.

---

### US-005: Font Embedding and Asset Extraction
**As a** Digital Archivist  
**I want to** ensure that custom fonts and vector graphics from the PDF are preserved in the HTML  
**So that** the document looks identical to the original regardless of system fonts.

*   **Acceptance Criteria**:
    *   [ ] Extract embedded PDF fonts and convert them to WOFF2 format.
    *   [ ] Generate a CSS `@font-face` manifest for each document.
    *   [ ] Convert PDF vector paths into optimized inline SVGs.
    *   [ ] Store extracted assets in an S3-compatible bucket with content-addressed naming (hashing).
*   **Test Scenarios**:
    *   **Scenario 1**: Convert a PDF with a rare corporate font. **Result**: HTML renders with the correct typeface; WOFF2 file found in network tab.
    *   **Scenario 2**: Convert a PDF with complex charts. **Result**: Charts are rendered as SVGs, not low-res JPEGs.
*   **Affected Components**: `ode-core` (Asset Pipeline), AWS S3.
*   **Story Estimation**: 
    *   Complexity: High
    *   Dependencies: US-001
*   **File References**: `src/engine/fonts.rs`, `src/engine/svg_renderer.rs`, `src/storage/s3.rs`
*   **Verification Steps**: Inspect generated HTML for `@font-face` rules and `<svg>` tags. Verify file existence in S3 via AWS CLI.

---

### US-006: Real-time Job Status via WebSockets
**As a** Frontend Engineer  
**I want to** receive real-time updates on the conversion progress  
**So that** I can show a progress bar to the user for large files.

*   **Acceptance Criteria**:
    *   [ ] Establish a WebSocket connection via Axum for a specific `jobId`.
    *   [ ] Emit events: `STARTED`, `PAGE_PROCESSED (n/total)`, `COMPLETED`, `FAILED`.
    *   [ ] Update Redis with current progress percentage every 500ms during processing.
*   **Test Scenarios**:
    *   **Scenario 1**: Monitor a 100-page conversion. **Result**: Receive 100 `PAGE_PROCESSED` events in sequence.
    *   **Scenario 2**: Disconnect and reconnect. **Result**: Client receives the latest state upon reconnection.
*   **Affected Components**: `ode-api` (WebSockets), `ode-worker`, Redis Pub/Sub.
*   **Story Estimation**: 
    *   Complexity: Medium
    *   Dependencies: US-002
*   **File References**: `src/api/ws/notifications.rs`, `src/worker/processor.rs`
*   **Verification Steps**: Use a WebSocket test client (e.g., Postman) to subscribe to `ws://api/v1/jobs/{id}/stream` and verify event sequence.

---

## Summary of Dependencies & Implementation Order

1.  **Phase 1 (Foundation)**: US-001 (Core Engine) + US-005 (Assets).
2.  **Phase 2 (Infrastructure)**: US-002 (API) + US-006 (WebSockets).
3.  **Phase 3 (Frontend)**: US-004 (React Viewer).
4.  **Phase 4 (Optimization)**: US-003 (Wasm).

## Verification Strategy
*   **Unit Testing**: `cargo test` for Rust logic; `vitest` for React components.
*   **Integration Testing**: Docker-compose environment running API, Redis, and Postgres.
*   **Visual Regression**: Playwright comparing PDF screenshots vs. rendered HTML screenshots.
*   **Performance**: Criterion.rs for benchmarking Rust conversion speed.