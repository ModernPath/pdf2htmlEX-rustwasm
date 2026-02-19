# Use Cases Documentation: HTMLRenderer

## Module Overview
The `src/HTMLRenderer` module provides the core functionality for converting PDF documents into HTML representations. It acts as an HTML Rendering Engine that processes PDF drawing commands, extracts fonts, handles text layout, and converts interactive elements like forms and links into web-compatible formats.

**User Roles and Personas**
*   **Status:** Not explicitly defined in code.
*   **Note:** The module functions as a backend processing engine (C++ library). There are no explicit user roles or permission checks defined within the provided code. The "user" is effectively the system or application invoking the `HTMLRenderer` class.

---

## Key User Journeys & Workflows

The following workflows describe the high-level processes executed by the system to transform PDF content into HTML.

### 1. Document Conversion Workflow
This is the primary workflow for transforming a PDF file into HTML.

*   **Trigger:** Invocation of the `HTMLRenderer::process` method (`src/HTMLRenderer/general.cc`).
*   **Process Flow:**
    1.  **Initialization:** The constructor `HTMLRenderer::HTMLRenderer` sets up global Poppler settings and configures state managers.
    2.  **Pre-processing:** The `pre_process` method calculates zoom factors and opens output streams for CSS.
    3.  **Page Iteration:** The `process` method iterates through each page of the PDF document.
    4.  **DPI Calculation:** For each page, the system calculates the DPI. This value is clamped to a maximum of `72 * 9000 / max(page_width, page_height)` to prevent excessive memory usage.
    5.  **Page Rendering:**
        *   `startPage` initializes state for the new page.
        *   Background rendering occurs (if `param.process_nontext` is true).
        *   Text, links, and forms are processed.
        *   `endPage` finalizes the page, dumping text, CSS, and closing tags.
    6.  **Output Handling:** If `split_pages` is enabled, each page is written to a separate file, and a frame is written to the main pages file.
    7.  **Resource Management:** The system monitors the total size of temporary files. Processing stops if the size exceeds `param.tmp_file_size_limit`.

### 2. Text Rendering & Layout Workflow
This workflow handles the conversion of PDF text streams into HTML text elements, ensuring visual fidelity.

*   **Trigger:** The `HTMLRenderer::drawString` method (`src/HTMLRenderer/text.cc`) is called during page processing.
*   **Process Flow:**
    1.  **State Check:** The system checks `HTMLRenderer::check_state_change` (`src/HTMLRenderer/state.cc`) to determine if the font, transformation matrix, or colors have changed. If so, a new line or text state is initiated.
    2.  **Font Handling:**
        *   Type 3 fonts and writing mode fonts are identified and routed to be rendered as images rather than text.
        *   Text rendering is skipped if the font render mode is >= 4.
    3.  **Character Processing:**
        *   The system iterates through the string, handling character spacing and word spacing.
        *   If `decompose_ligature` is enabled, ligatures are broken down into individual Unicode characters.
        *   If `space_as_offset` is enabled, space characters are treated as padding/offset rather than rendered characters.
    4.  **Position Calculation:** Character width and height are calculated. If calculated as zero, they default to 0.001 to prevent rendering errors.
    5.  **Visibility Check:** `is_char_covered` checks if a character is obscured by a background element (like an image). If covered, it may be skipped to fail safely.

### 3. Interactive Element Conversion Workflow
This workflow converts PDF interactive features (links and forms) into HTML equivalents.

*   **Trigger:**
    *   Links: `HTMLRenderer::processLink` (`src/HTMLRenderer/link.cc`).
    *   Forms: `HTMLRenderer::process_form` (`src/HTMLRenderer/form.cc`).
*   **Process Flow (Links):**
    1.  **Action Parsing:** `get_linkaction_str` determines the target (internal page jump or external URI) based on the `LinkAction` type.
    2.  **Destination Formatting:** `get_linkdest_detail_str` generates a string representation of the destination (page number and coordinates) for JavaScript parsing.
    3.  **Style Mapping:** PDF border styles are mapped to CSS styles (e.g., "Beveled" becomes "outset", "Inset" becomes "inset").
    4.  **Rendering:** An HTML `<a>` tag wrapping a positioned `<div>` is generated. A transparent background color (`rgba(255,255,255,0.000001)`) is applied as a fix for Internet Explorer compatibility.
*   **Process Flow (Forms):**
    1.  **Widget Iteration:** The system iterates through form widgets on the current page.
    2.  **Scaling:** Coordinates and dimensions are scaled by a zoom factor.
    3.  **Element Generation:**
        *   **Text Fields:** An HTML input is generated. The font size is calculated as half the field's height.
        *   **Buttons:** An HTML button/div is generated. Width and height are increased by 3 pixels.
    4.  **Error Handling:** If an unsupported field type is encountered, an error message is written to `cerr`, and execution continues.

---

## Business Process Flows

### Font Extraction and Embedding
*   **Description:** The system extracts font data from the PDF to ensure the HTML output looks correct.
*   **Logic:**
    *   `dump_embedded_font` (`src/HTMLRenderer/font.cc`) extracts raw font data streams.
    *   File extensions are determined by the 'Subtype' in the FontDescriptor (e.g., 'Type1C' -> `.cff`, 'OpenType' -> `.otf`).
    *   **Type 3 Font Special Handling:** Type 3 fonts are rendered to SVG files using Cairo. They are scaled so the longer edge of their bounding box is 100.0 units (`GLYPH_DUMP_EM_SIZE`).
    *   The extracted fonts are embedded into the HTML output via `embed_font`.

### State Management & Synchronization
*   **Description:** The system tracks PDF graphics state changes to optimize the HTML structure.
*   **Logic:**
    *   Methods like `updateFont`, `updateCTM`, and `updateRender` (`src/HTMLRenderer/state.cc`) track changes to fonts, transformations, and colors.
    *   **Matrix Rescaling:** Font size and transformation matrices are rescaled to keep the matrix as close to the identity matrix as possible for CSS compatibility.
    *   **Negative Size Handling:** Negative font sizes (indicating flipped pages) are inverted to positive values, and the transformation matrix is adjusted.
    *   **Text Merging:** Text is only merged into the current line if transformation matrices are proportional (parallel text).

---

## Feature Descriptions (User Perspective)

Based on the implemented code, the system provides the following features:

*   **PDF to HTML Conversion:** Converts entire PDF documents into HTML format, preserving layout and content.
*   **Page Splitting:** Offers the capability to split PDF pages into separate HTML files (controlled by `split_pages` parameter).
*   **Background Rendering:** Can process non-text background elements. This feature can be disabled via `param.process_nontext`.
*   **Link Preservation:** Converts internal and external PDF links into clickable HTML anchor tags with correct positioning.
*   **Form Conversion:** Converts PDF text fields and buttons into HTML input elements and buttons.
*   **Ligature Decomposition:** Provides an option to decompose ligatures into individual characters for better text handling.
*   **Resource Optimization:** Automatically clamps DPI and monitors temporary file sizes to prevent system overload during conversion.

---

## Edge Cases and Error Handling

The system implements specific handling for the following edge cases and error scenarios:

*   **Resource Limits:**
    *   **Temp File Size:** The conversion process halts if the total size of temporary files exceeds `param.tmp_file_size_limit` (`src/HTMLRenderer/general.cc`).
    *   **DPI Clamping:** Prevents excessive memory usage by clamping the DPI calculation based on page dimensions.
*   **Rendering Constraints:**
    *   **Zero Dimensions:** Character width and height default to 0.001 if calculated as zero to prevent rendering errors (`src/HTMLRenderer/text.cc`).
    *   **Obscured Text:** The `is_char_covered` function returns `true` (error state) if an index is out of bounds, failing safely.
*   **Unsupported Features:**
    *   **Form Fields:** Unsupported form field types trigger an error message to `cerr` but do not crash the process (`src/HTMLRenderer/form.cc`).
    *   **Link Actions:** Unimplemented link actions (specifically `actionGoToR` and `actionLaunch`) result in a warning being logged to `cerr` (`src/HTMLRenderer/link.cc`).
    *   **Unknown Border Styles:** Unknown PDF border styles result in a warning log to `cerr`.
*   **File I/O:**
    *   Font dumping operations use try-catch blocks and throw strings for file I/O failures (`src/HTMLRenderer/font.cc`).

---

## Sequence Diagrams

### Sequence: Main Document Conversion
This sequence illustrates the high-level flow of converting a document.

```text
Caller
  |
  |--> HTMLRenderer::process()
        |
        |--> pre_process() [Calculates zoom, opens CSS stream]
        |
        |--> Loop (Pages)
              |
              |--> startPage() [Resets state]
              |
              |--> (Internal PDF Rendering Calls)
              |      |
              |      |--> drawString() [Text processing]
              |      |--> processLink() [Link processing]
              |      |--> process_form() [Form processing]
              |      |--> drawImage() [Image handling]
              |
              |--> endPage() [Dumps CSS, text, forms]
              |
              |--> Check: Is tmp_file_size > limit?
              |      |--> (If yes, Stop Processing)
        |
        |--> (Finish)
```

### Sequence: Text State Change Management
This sequence shows how the renderer decides when to change the HTML text structure.

```text
PDF Engine
  |
  |--> updateFont() or updateCTM()
        |
        |--> HTMLRenderer (State Updated)
        |
  |--> drawString()
        |
        |--> check_state_change()
              |
              |--> Analyze: Did Font change?
              |      |--> (If yes, and Type 3 involved -> Force New Line)
              |
              |--> Analyze: Are matrices proportional?
              |      |--> (If no, Start New Line)
              |
              |--> Analyze: Is font size negative?
              |      |--> (If yes, Invert size & Adjust Matrix)
              |
              |--> (Output HTML Span/Line)
```