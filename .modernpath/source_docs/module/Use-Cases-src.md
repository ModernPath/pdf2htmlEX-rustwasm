# Use Cases Documentation: `src`

## Overview
The `src` module provides the core functionality for a command-line PDF to HTML conversion utility (`pdf2htmlEX`). It orchestrates the end-to-end workflow of parsing command-line arguments, analyzing PDF document structure, detecting text visibility, managing CSS state, and generating optimized HTML output.

**User Roles and Personas:** Not explicitly defined in code. The module exposes a command-line interface (CLI) for execution.

---

## Key Use Cases

### UC-01: Execute PDF to HTML Conversion
**Description:** The primary workflow where the user initiates the conversion of a PDF document to HTML format via the command line.

**Workflow Steps:**
1.  **Parse Configuration:** The system parses command-line arguments to configure conversion parameters (e.g., DPI, zoom, embedding options).
    *   *Implementation:* `pdf2htmlEX::parse_options`, `ArgParser::parse`.
2.  **Prepare Environment:** The system creates a unique temporary directory required for intermediate processing files.
    *   *Implementation:* `pdf2htmlEX::prepare_directories`.
3.  **Preprocess Document:** The system performs an initial scan of the PDF to collect metadata (font usage, page dimensions).
    *   *Implementation:* `Preprocessor::process`.
4.  **Render Output:** The system processes the document pages to generate the HTML content.
    *   *Implementation:* Delegated to `HTMLRenderer` (referenced in `pdf2htmlEX.cc`).
5.  **Cleanup:** The system removes temporary files and directories if configured to do so.
    *   *Implementation:* `TmpFiles` destructor.

**Business Rules:**
*   The application must exit with a failure status if the temporary directory cannot be created (`pdf2htmlEX.cc`).
*   Embedding options (CSS, fonts, images) can be toggled via a single string argument (`pdf2htmlEX.cc`).

---

### UC-02: Analyze Document Structure (Preprocessing)
**Description:** Before rendering, the system scans the PDF document to gather necessary metadata for optimization and correct rendering.

**Workflow Steps:**
1.  **Iterate Pages:** The system loops through the specified page range.
    *   *Implementation:* `Preprocessor::process`.
2.  **Track Dimensions:** The system calculates the maximum width and height encountered across all pages.
    *   *Implementation:* `Preprocessor::startPage`.
3.  **Map Font Usage:** The system records which character codes (glyphs) are used for each font to optimize font subsetting.
    *   *Implementation:* `Preprocessor::drawChar`.

**Business Rules:**
*   PDF documents must be scanned twice to handle complexity and collect necessary metadata (`Preprocessor.h`).
*   Type 3 fonts should not be interpreted during preprocessing (`Preprocessor.h`).
*   Rendering operations must be clipped to the crop box (`Preprocessor.h`).

---

### UC-03: Detect Text Visibility (Occlusion)
**Description:** The system determines if text characters are visually obscured by other graphical elements (like images or shapes) to handle them correctly in the output.

**Workflow Steps:**
1.  **Register Characters:** The system registers the bounding box of every drawn character, initially marking it as visible.
    *   *Implementation:* `CoveredTextDetector::add_char_bbox`.
2.  **Trace Graphics:** The system intercepts drawing operations for non-character elements (strokes, fills).
    *   *Implementation:* `DrawingTracer::stroke`, `DrawingTracer::fill`.
3.  **Detect Intersection:** The system checks if the bounding box of a non-character element intersects with existing character boxes.
    *   *Implementation:* `CoveredTextDetector::add_non_char_bbox`.
4.  **Update Status:** If an intersection occurs, the character is marked as covered.
    *   *Implementation:* `CoveredTextDetector::add_non_char_bbox`.

**Business Rules:**
*   Strokes or fills with opacity less than 0.5 are ignored for occlusion purposes (`DrawingTracer.cc`).
*   A character is considered covered if its bounding box intersects with a non-character graphic drawn after it (`CoveredTextDetector.h`).
*   If a character is partially covered and visibility correction mode is 2, the background rendering DPI is increased (`CoveredTextDetector.cc`).

---

### UC-04: Generate Optimized HTML Output
**Description:** The system converts the internal representation of PDF text and graphics into structured HTML and CSS, optimizing for file size and visual fidelity.

**Workflow Steps:**
1.  **Aggregate Text Lines:** The system collects text data into line objects.
    *   *Implementation:* `HTMLTextPage::open_new_line`.
2.  **Manage CSS States:** The system assigns unique IDs to style values (fonts, colors, transforms) to avoid duplication in the CSS.
    *   *Implementation:* `StateManager::install`.
3.  **Optimize Structure:** The system merges adjacent text states and optimizes whitespace.
    *   *Implementation:* `HTMLTextLine::optimize`.
4.  **Serialize Content:** The system writes the final HTML markup, including handling clipping regions and transparency.
    *   *Implementation:* `HTMLTextPage::dump_text`, `HTMLTextLine::dump_text`.

**Business Rules:**
*   Offsets that are very small (within `h_eps`) are ignored and not rendered (`HTMLTextLine.cc`).
*   A clip region is only rendered as a `div` if its dimensions do not exactly match the full page dimensions (`HTMLTextPage.cc`).
*   Text characters covered by a clipping region are rendered as transparent (invisible) but still occupy space (`HTMLTextLine.cc`).

---

### UC-05: Manage Temporary Resources
**Description:** The system tracks intermediate files created during conversion and ensures they are cleaned up according to configuration.

**Workflow Steps:**
1.  **Registration:** As files are created, they are added to a tracking registry.
    *   *Implementation:* `TmpFiles::add`.
2.  **Size Calculation:** The system calculates the total disk space used by temporary files.
    *   *Implementation:* `TmpFiles::get_total_size`.
3.  **Deletion:** Upon destruction of the manager object, the system removes all tracked files and the temporary directory.
    *   *Implementation:* `TmpFiles::clean`.

**Business Rules:**
*   Temporary files are only tracked and cleaned if the `clean_tmp` parameter is true (`TmpFiles.cc`).
*   The temporary directory is removed only after all individual files within it have been removed (`TmpFiles.cc`).

---

## Error Handling and Edge Cases

*   **Argument Validation:** If a command-line option requires an argument but none is provided, or if the type conversion fails, the system throws an exception (`ArgParser::ArgEntry::parse`).
*   **Missing Styles:** If text is found without a corresponding style state during HTML generation, a warning is printed to `stderr` (`HTMLTextLine.cc`).
*   **Directory Creation:** If the temporary directory creation fails, the application outputs an error message derived from `strerror(errno)` and exits (`pdf2htmlEX.cc`).
*   **State Consistency:** The `StringFormatter` asserts that the internal buffer is not reused (`buf_cnt` must be 0) before a new formatting operation begins (`StringFormatter.cc`).

## Common Usage Patterns

*   **Two-Pass Processing:** The system consistently employs a two-pass strategy: a preprocessing pass to collect metadata (`Preprocessor`) followed by a rendering pass to generate output.
*   **State Stack Management:** Both the graphics state (`DrawingTracer`) and the HTML text state (`HTMLTextLine`) utilize stack-based patterns to manage transformations and style contexts.
*   **Callback Integration:** The system heavily utilizes callbacks (e.g., in `DrawingTracer` and `CoveredTextDetector`) to decouple the detection of events (like drawing operations) from the processing logic.