# ODE (Oxidized Document Engine)

A modern, high-performance PDF to HTML conversion engine written in Rust. ODE is a clean-room rewrite of [pdf2htmlEX](https://github.com/pdf2htmlEX/pdf2htmlEX), the pioneering C++ tool created by Lu Wang that converts PDF documents into high-fidelity HTML with text, images, and styling preserved.

## Acknowledgments

This project is inspired by and builds upon the ideas of **pdf2htmlEX**, originally created by [Lu Wang](https://github.com/coolwanglu) and [contributors](https://github.com/pdf2htmlEX/pdf2htmlEX/graphs/contributors). While ODE is a complete rewrite in Rust (not a fork), we gratefully acknowledge the foundational work of the pdf2htmlEX project and maintain the same GPLv3+ license.

## Quick Start

```bash
# Build
cargo build --release --package ode-api

# Run (standalone mode — no database or external services needed)
PORT=3000 ./target/release/ode-api

# Open the web UI
open http://localhost:3000/ui
```

Or with Docker:

```bash
docker build -t ode .
docker run -p 3000:3000 ode
```

## API

```bash
# Convert a PDF (returns HTML)
curl -X POST http://localhost:3000/v1/convert-sync -F "file=@document.pdf" -o output.html

# Health check
curl http://localhost:3000/health
```

## Project Structure

```
crates/
  ode-core/       # Core conversion engine (PDF parsing, rendering, HTML generation)
  ode-api/        # Axum HTTP server with web UI
  ode-worker/     # Async job worker (for full mode with Redis)
  ode-wasm/       # WebAssembly wrapper (planned)
```

### Core Engine (ode-core)

- **PDF Parser**: Custom zero-dependency PDF parser with XRef table, object streams, page tree traversal with resource inheritance
- **Text Extraction**: Content stream parsing, CTM transforms, ToUnicode CMap decoding (single-byte and multi-byte)
- **Image Extraction**: DCTDecode (JPEG), JPXDecode (JPEG2000), FlateDecode (raw pixels with PNG encoding)
- **HTML/CSS Rendering**: Absolute-positioned text spans, inline base64 images, background color detection
- **Font Processing**: Font extraction framework with content-addressed storage

### API Server (ode-api)

- **Standalone mode** (default): Single binary, no external dependencies. Just PDF conversion and web UI.
- **Full mode** (`ODE_MODE=full`): PostgreSQL, Redis, S3 for async jobs, user auth, API keys, rate limiting.

## Server Modes

| Mode | Env Var | Requires | Endpoints |
|------|---------|----------|-----------|
| Standalone (default) | — | Nothing | `/ui`, `/v1/convert-sync`, `/health` |
| Full | `ODE_MODE=full` | PostgreSQL, Redis, S3 | All above + `/v1/convert`, `/auth/*`, `/v1/profiles/*` |

## Building

```bash
cargo build --release                     # Build all crates
cargo test -p ode-core                    # Run core tests (130+)
cargo test                                # Run all tests
```

## License

ODE is licensed under the **GNU General Public License v3.0 or later** (GPLv3+), the same license as the original pdf2htmlEX.

See [LICENSE](LICENSE) for the full text.

### Original Work

pdf2htmlEX — Copyright (c) 2012-2014 Lu Wang and contributors
https://github.com/pdf2htmlEX/pdf2htmlEX

## Maintainer

Pasi Vuorio — [pasi@modernpath.ai](mailto:pasi@modernpath.ai)
