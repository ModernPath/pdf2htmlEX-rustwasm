# Interfaces Documentation: src/HTMLRenderer

## Overview
The `src/HTMLRenderer` module provides a C++ class interface designed to convert PDF documents into HTML format. It acts as a bridge between the external **Poppler** PDF rendering library and internal HTML generation logic.

The module does not expose REST APIs, WebSockets, or network protocols. Instead, its interface consists of a C++ class (`HTMLRenderer`) that integrates with Poppler via the **`OutputDev`** interface (Inheritance pattern). The primary integration point is the consumption of PDF drawing commands and state changes from Poppler and the production of file-based HTML/CSS output.

## Core Class Interface: `HTMLRenderer`

The `HTMLRenderer` class (defined in `HTMLRenderer.h`) serves as the main API for the subsystem. It inherits from Poppler's `OutputDev`, implementing virtual hooks to intercept the rendering process.

### Lifecycle & Orchestration Methods
These methods control the high-level workflow of the conversion process.

*   **`HTMLRenderer(params)`** (Constructor)
    *   **Purpose:** Initializes the renderer, sets up global Poppler settings, and configures internal state managers.
    *   **Side Effects:** Modifies global state and internal configuration.
*   **`process()`**
    *   **Purpose:** Main entry point to process a PDF document. Iterates through pages, calculates DPI, and manages background rendering.
    *   **Side Effects:** Generates file output (HTML/CSS), modifies internal state.
*   **`startPage()`**
    *   **Purpose:** Called when a new page begins rendering. Initializes state for the page.
    *   **Side Effects:** Resets text detectors and tracers.
*   **`endPage()`**
    *   **Purpose:** Called when a page finishes rendering. Finalizes the current page by dumping text, CSS, form data, and links.
    *   **Side Effects:** Writes closing HTML tags to file output.

### Drawing & Content Hooks
These methods are called by the Poppler engine to render specific content types.

*   **`drawString(GfxState *state, GooString *s)`**
    *   **Purpose:** Processes raw PDF text strings, calculating positions and handling font specifics.
    *   **Side Effects:** Appends text data to the internal HTML output structure.
*   **`drawImage(GfxState *state, Object *ref, Stream *str, int width, int height, GfxImageColorMap *colorMap, ...)`**
    *   **Purpose:** Handles the drawing of standard PDF images.
    *   **Side Effects:** Invokes internal tracer; delegates to base `OutputDev` implementation.
*   **`drawSoftMaskedImage(GfxState *state, Object *ref, Stream *str, ...)`**
    *   **Purpose:** Handles the drawing of PDF images with soft masks (alpha channels).
    *   **Side Effects:** Invokes internal tracer; delegates to base `OutputDev` implementation.
*   **`stroke(GfxState *state)`**
    *   **Purpose:** Handles the stroking of a path.
    *   **Side Effects:** Delegates to the internal `tracer` object.
*   **`fill(GfxState *state)`**
    *   **Purpose:** Handles the filling of a path.
    *   **Side Effects:** Delegates to the internal `tracer` object.
*   **`eoFill(GfxState *state)`**
    *   **Purpose:** Handles the even-odd rule filling of a path.
    *   **Side Effects:** Delegates to the internal `tracer` object.
*   **`processLink(Link *link, Catalog *catalog)`**
    *   **Purpose:** Renders PDF link annotations as HTML `<a>` tags.
    *   **Side Effects:** Generates DOM elements (divs/anchors) with CSS positioning.
*   **`process_form(AnnotWidget *widget)`**
    *   **Purpose:** Converts PDF form widgets into HTML input elements.
    *   **Side Effects:** Writes HTML tags to output stream.

### State Management Hooks
These methods track changes in the PDF graphics state (GfxState).

*   **`updateAll(GfxState *state)`**
    *   **Purpose:** Marks all states as changed and updates text position.
*   **`updateCTM(GfxState *state, double m11, double m12, double m21, double m22, double m31, double m32)`**
    *   **Purpose:** Updates the Current Transformation Matrix (CTM).
*   **`updateTextPos(GfxState *state)`**
    *   **Purpose:** Updates current text coordinates (`cur_tx`, `cur_ty`).
*   **`updateFont(GfxState *state)`**
    *   **Purpose:** Marks the font state as changed.
*   **`updateRender(GfxState *state)`**
    *   **Purpose:** Marks fill and stroke colors as changed.
*   **`clip(GfxState *state)`**
    *   **Purpose:** Marks clipping state as changed.
*   **`saveState(GfxState *state)`**
    *   **Purpose:** Saves the current graphics state to the stack.
*   **`restoreState(GfxState *state)`**
    *   **Purpose:** Restores the previous graphics state.

### Transparency & Grouping
*   **`beginTransparencyGroup(GfxState *state, double *bbox, GfxColorSpace *blendingColorSpace, ...)`**
    *   **Purpose:** Signals the start of a transparency group.
*   **`endTransparencyGroup(GfxState *state)`**
    *   **Purpose:** Signals the end of a transparency group.

## External Integrations

### Poppler PDF Library
The module integrates tightly with the Poppler library by implementing the `OutputDev` interface.
*   **Integration Type:** Inheritance (Observer Pattern).
*   **Key Dependencies:** `OutputDev`, `GfxState`, `PDFDoc`, `GfxFont`, `Link`, `Annot`, `Page`, `Form`, `GlobalParams`.
*   **Protocol:** Virtual method overrides (hooks) called by Poppler during the PDF parsing phase.

### Cairo Graphics Library
Used for specific font rendering tasks.
*   **Integration Type:** Direct Library Calls.
*   **Location:** `font.cc`.
*   **Key Dependencies:** `cairo`, `cairo-ft`, `cairo-svg`.
*   **Purpose:** Renders Type 3 font glyphs to SVG files.

### File System
The module interacts directly with the OS file system for output and temporary storage.
*   **Integration Type:** Standard C++ File I/O.
*   **Operations:** Writing HTML/CSS output streams, dumping embedded fonts to temporary files, creating SVG files for Type 3 fonts.

## Internal Component Dependencies

The `HTMLRenderer` delegates specific tasks to other internal subsystems:

*   **`BackgroundRenderer`**: Used in `general.cc` to handle non-text background rendering.
*   **`CoveredTextDetector`**: Used in `general.cc` and `text.cc` to detect if text is obscured by other elements.
*   **`DrawingTracer`**: Used in `draw.cc`, `image.cc`, and `state.cc` to record or trace drawing commands.
*   **`Param`**: Configuration parameters used to control rendering behavior (e.g., `split_pages`, `process_nontext`).
*   **`HTMLTextLine`**: Used in `general.cc` to manage text line structures.

## Error Handling

The module employs a mix of standard output streams and exceptions for error handling, though no formal error class hierarchy is explicitly defined in the headers.

*   **Logging:** Uses `std::cerr` to output warnings for non-critical issues (e.g., non-zero character origins in `text.cc`, unimplemented link actions in `link.cc`, unsupported form fields in `form.cc`).
*   **Exceptions:** Throws string literals (`throw string`) for critical failures, specifically file I/O errors or unsupported formats in `font.cc` and `general.cc`.
*   **Safe Returns:** Functions like `is_char_covered` in `text.cc` return boolean flags to indicate failure states safely without crashing.
*   **External Library Configuration:** Configures Poppler's "quiet mode" to suppress error messages if debug mode is disabled (`general.cc`).