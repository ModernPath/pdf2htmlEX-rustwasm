# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is the **Oxidized Document Engine (ODE)** — a Rust rewrite of pdf2htmlEX, a C/C++ tool that converts PDF documents into high-fidelity HTML/CSS/SVG. The project is managed by the ModernPath CLI, which provides the legacy source code, architecture specs, and transformation guides.

**Current state**: Pre-implementation. No Rust code has been written yet. The `.modernpath/` directory contains the legacy C++ source, auto-generated documentation, and transformation specifications.

## Repository Layout

```
AGENTS.md                    # ModernPath transformation guide (file-by-file instructions)
.modernpath/
  source/                    # Legacy pdf2htmlEX C/C++ source code (read-only reference)
  source_docs/               # Auto-generated docs about the legacy system (by tier/angle)
  docs/                      # Target architecture documentation
  specs/
    architecture/            # System architecture, component specs, integration, deployment
    modernization/           # Migration strategy, code mapping, API translation
    requirements/            # Epics: core features, user management, data management
    database/                # Schema changes, ERD, API contracts
    testing/                 # Unit, integration, e2e, performance, API test plans
```

## Target Architecture (ODE)

A Rust workspace with five crates/packages, built in phases:

| Crate | Purpose | Key Dependencies |
|-------|---------|-----------------|
| `crates/ode-core` | PDF parsing, font extraction, HTML/SVG generation | `lopdf`/`pdf-rs`, `font-kit`, `resvg`, `tiny-skia` |
| `crates/ode-api` | Axum REST API (job submission, status, retrieval) | `axum`, `tokio`, `sqlx`, `serde` |
| `crates/ode-worker` | Async Redis job consumer, S3 storage | `tokio`, `deadpool-redis`, `aws-sdk-s3` |
| `crates/ode-wasm` | WebAssembly wrapper for in-browser conversion | `wasm-bindgen`, `js-sys` |
| `packages/ode-react` | React document viewer component | React, Radix UI, Tailwind CSS |

### Build Commands (once code exists)

```bash
cargo build --release                     # Build all Rust crates
cargo test                                # Run all Rust tests
cargo test -p ode-core                    # Test a single crate
cargo test -p ode-core -- test_name       # Run a specific test
wasm-pack build crates/ode-wasm --target web  # Build Wasm module
docker-compose up -d                      # Start PostgreSQL + Redis
cargo run --bin ode-api                   # Run the API server
```

### Core API

```rust
// ode-core entry point
pub fn convert_pdf(data: Vec<u8>, options: ConvertOptions) -> Result<OutputBundle, OdeError>

// REST endpoints
POST /v1/convert        // Upload PDF, returns job_id
GET  /v1/status/:job_id // Job processing state
GET  /v1/documents/:id  // Retrieve converted HTML/assets
```

## Transformation Workflow

Follow AGENTS.md for file-by-file transformation. The general process:

1. Read specs in `.modernpath/specs/` for requirements
2. Study legacy source in `.modernpath/source/` and docs in `.modernpath/source_docs/`
3. Check target architecture in `.modernpath/docs/`
4. Use `modernpath read-file <path>` CLI for additional legacy source context
5. Write modernized Rust code in the target crate structure

### Phasing

- **Phase 1 (Core Engine)**: `ode-core` — PDF parsing, graphics state, font handling, HTML/SVG output
- **Phase 2 (API & Orchestration)**: `ode-api` + `ode-worker` — Axum server, Redis queue, S3 storage
- **Phase 3 (Wasm & UI)**: `ode-wasm` + `ode-react` — Browser-side conversion, React viewer
- **Phase 4 (Production)**: Terraform, EKS, monitoring (Prometheus/Grafana/OpenTelemetry)

## Key Legacy Subsystems → Rust Mapping

| Legacy (C++) | Target (Rust) | Notes |
|-------------|---------------|-------|
| `src/HTMLRenderer/` | `ode-core/src/renderer/` | Text positioning, state tracking, CSS generation |
| `src/BackgroundRenderer/` | `ode-core/src/render/background.rs` | SVG/raster backgrounds via `resvg`/`tiny-skia` |
| `src/util/ffw.c` (FontForge) | `ode-core/src/fonts/` | Replace with `ttf-parser`/`font-kit`/`woff2` crate |
| `src/pdf2htmlEX.cc` (main/CLI) | `ode-api` + `ode-worker` | CLI → async microservice |
| Poppler (PDF parsing) | `lopdf` or `pdf-rs` | Memory-safe PDF tree traversal |
| Cairo (SVG) / Splash (raster) | `resvg` / `tiny-skia` | Pure Rust rendering |

## Acceptance Criteria

- Zero `unsafe` blocks in `ode-core` unless strictly required for audited FFI
- 99% visual parity with pdf2htmlEX output (Playwright pixel-diff < 0.1%)
- Server conversion of 10-page PDF < 2s (p95)
- Wasm conversion of single page < 500ms in-browser
- Generated HTML passes WCAG 2.1 AA (axe-core)
