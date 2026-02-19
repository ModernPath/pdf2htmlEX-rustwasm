# ODE (Oxidized Document Engine)

A high-performance Rust-based PDF to HTML conversion engine.

## Project Structure

```
crates/
  ode-core/src/           # Core conversion engine
    lib.rs                # Main entry point (convert_pdf function)
    parser/mod.rs         # PDF document parsing
    renderer/mod.rs       # HTML/CSS output generation
    render/mod.rs         # Graphics state tracking
    render/state.rs       # State management (fonts, colors, transforms)
    fonts/mod.rs          # Font extraction framework
    types/color.rs        # Color handling
    util/math.rs          # Matrix transformations, bounding boxes
    config.rs             # Conversion configuration
    error.rs              # Error types with thiserror
```

## Implemented Components

### Core Library (ode-core)
- **PDF Parsing**: Uses `pdf` crate to parse PDF documents and extract page metadata
- **Configuration**: Full configuration mapping from legacy Param.h
- **Error Handling**: Comprehensive error types with `thiserror`
- **Color Processing**: RGB color handling with distance calculation and CSS output
- **Math Utilities**: Transform matrices, bounding boxes, geometric operations
- **State Tracking**: Graphics state management (fonts, colors, transformations)
- **HTML Generation**: Page structure with absolute positioning
- **Font Framework**: Structure for font extraction (WOFF2 support requires additional deps)

## Usage Example

```rust
use ode_core::{convert_pdf, ConversionConfig};

let pdf_data = std::fs::read("document.pdf")?;
let config = ConversionConfig::default();

let output = convert_pdf(&pdf_data, config)?;
for page in output.pages {
    println!("Page {} HTML:\n{}", page.page_number, page.html);
}
```

## Building

```bash
cargo build --package ode-core
```

## Status

✅ Core engine structure
✅ PDF parsing foundation
✅ Configuration system
✅ Error handling
✅ Math utilities
✅ Graphics state tracking
✅ HTML/CSS generation
⏳ Full text extraction (next iteration)
⏳ Font embedding with WOFF2 (requires dependency resolution)
⏳ Visual regression tests