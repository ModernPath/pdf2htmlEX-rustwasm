# Interfaces Documentation: `src`

## Overview
The `src` module serves as the core processing engine for a PDF to HTML conversion utility. It does not expose REST, GraphQL, or WebSocket APIs. Instead, its interfaces are primarily **Command Line Interfaces (CLI)**, **C++ Class APIs** for internal component interaction, and **Integration Points** with external graphics libraries (Poppler and Cairo).

The module orchestrates a pipeline that parses command-line arguments, preprocesses PDF documents via the Poppler library, traces drawing operations via Cairo, and generates optimized HTML/CSS output.

## External Integrations

### Poppler Library Integration
The module integrates with the Poppler PDF rendering library to access document structure and content.

*   **Integration Point:** `src/Preprocessor.h` / `src/Preprocessor.cc`
*   **Mechanism:** The `Preprocessor` class inherits from Poppler's `OutputDev` class.
*   **Implemented Virtual Methods (Callbacks):**
    *   `upsideDown()`: Returns `false` to configure coordinate system orientation.
    *   `useDrawChar()`: Returns `true` to enable character drawing callbacks.
    *   `interpretType3Chars()`: Returns `false` to disable Type 3 font interpretation during preprocessing.
    *   `needNonText()`: Returns `false` to indicate non-text graphical elements are not needed in this pass.
    *   `needClipToCropBox()`: Returns `true` to enforce clipping to the PDF's crop box.
    *   `drawChar(...)`: Callback invoked when a character is drawn to track font usage.
    *   `startPage(...)`: Callback invoked at the start of each page to track dimensions.
*   **Purpose:** Performs a preliminary scan to collect font usage metrics (character codes) and maximum page dimensions before the main rendering process.

### Cairo Library Integration
The module integrates with the Cairo graphics library to perform geometric analysis and trace drawing operations.

*   **Integration Point:** `src/DrawingTracer.h` / `src/DrawingTracer.cc`
*   **Mechanism:** The `DrawingTracer` class utilizes Cairo recording surfaces (`cairo_surface_t`, `cairo_t`) to replay and analyze PDF drawing commands.
*   **Key Operations:**
    *   `reset(...)`: Initializes the Cairo recording surface.
    *   `update_ctm(...)`: Updates the Current Transformation Matrix (CTM) stack.
    *   `clip(...)`: Applies clipping paths to the Cairo context.
    *   `stroke(...)`: Traces stroke operations, calculating bounding boxes.
    *   `fill(...)`: Traces fill operations, calculating bounding boxes.
*   **Purpose:** Calculates bounding boxes of drawn elements and detects occlusion (text visibility) by analyzing geometry.

## Command Line Interface (CLI)

The primary external interface for the application is the command line, managed by the entry point and argument parser.

### Entry Point
*   **File:** `src/pdf2htmlEX.cc`
*   **Function:** `main` (implied by file summary)
*   **Workflow:**
    1.  Initializes global parameters.
    2.  Parses command-line arguments using `ArgParser`.
    3.  Prepares necessary directories (temporary directory creation).
    4.  Delegates conversion to `HTMLRenderer`.

### Argument Parser Interface
*   **File:** `src/ArgParser.h` / `src/ArgParser.cc`
*   **Class:** `ArgParser`
*   **Public API:**
    *   `add(...)`: Registers command-line arguments. Supports linking to variables or callback functions (`ArgParserCallBack`). Handles short and long options.
    *   `parse(...)`: Executes parsing of the argument vector using `getopt_long`. Throws `std::string` exceptions on missing or invalid arguments.
    *   `show_usage(...)`: Generates and prints usage documentation to an output stream.
*   **Specific Argument Handling:**
    *   **Embed Options:** The `embed_parser` function handles the `--embed` flag, parsing a string argument to toggle embedding flags for CSS, fonts, and images.

## Internal Component APIs

### Rendering & Tracing APIs

#### DrawingTracer API
*   **File:** `src/DrawingTracer.h`
*   **Purpose:** Intercepts drawing operations and notifies registered callbacks.
*   **Key Methods:**
    *   `draw_char(...)`: Handles character drawing events.
    *   `draw_image(...)`: Handles image drawing events.
    *   `on_non_char_drawn`, `on_char_drawn`, `on_char_clipped`: Callbacks triggered during the rendering pipeline to notify higher-level logic of geometric events.

#### CoveredTextDetector API
*   **File:** `src/CoveredTextDetector.h`
*   **Purpose:** Detects text occlusion by analyzing bounding boxes.
*   **Key Methods:**
    *   `reset()`: Clears internal state for a new page.
    *   `add_char_bbox(...)`: Registers a character's bounding box.
    *   `add_char_bbox_clipped(...)`: Registers a character's bounding box with visibility flags.
    *   `add_non_char_bbox(...)`: Registers a non-character graphic (e.g., shape) and updates character visibility status if intersections occur.
    *   `get_chars_covered()`: Returns a vector of booleans indicating coverage status.

### HTML Generation APIs

#### StateManager API
*   **File:** `src/StateManager.h`
*   **Purpose:** Manages CSS state generation using a Flyweight pattern to ensure unique class generation.
*   **Template Classes:** `StateManager<double>`, `StateManager<Matrix>`, `StateManager<Color>`.
*   **Concrete Implementations:** `FontSizeManager`, `ColorManager`, `TransformMatrixManager`, etc.
*   **Key Methods:**
    *   `install(...)`: Registers a value (double, Matrix, Color), returning an ID. Handles deduplication based on epsilon comparison or hashing.
    *   `dump_css(...)`: Generates CSS rules for all installed values to an output stream.

#### HTMLTextLine API
*   **File:** `src/HTMLTextLine.h`
*   **Purpose:** Manages the conversion of a single line of PDF text into HTML.
*   **Key Methods:**
    *   `append_unicodes(...)`: Adds Unicode characters to the line buffer.
    *   `append_offset(...)`: Adds horizontal whitespace/offsets.
    *   `append_state(...)`: Appends a new text style state (font, color).
    *   `dump_text(...)`: Serializes the line content to an output stream as HTML.

#### HTMLTextPage API
*   **File:** `src/HTMLTextPage.h`
*   **Purpose:** Aggregates text lines and manages page-level clipping.
*   **Key Methods:**
    *   `open_new_line(...)`: Instantiates a new `HTMLTextLine`.
    *   `clip(...)`: Records a clipping region.
    *   `dump_text(...)`: Orchestrates the generation of HTML for the entire page.
    *   `dump_css(...)`: Outputs CSS styles (currently a placeholder in implementation).

### Utility APIs

#### StringFormatter API
*   **File:** `src/StringFormatter.h`
*   **Purpose:** Provides a reusable buffer for printf-style string formatting.
*   **Key Method:**
    *   `operator()(...)`: Formats a string into the internal buffer, returning a `GuardedPointer` for safe access.

#### Base64Stream API
*   **File:** `src/Base64Stream.h`
*   **Purpose:** Encodes binary streams into Base64.
*   **Key Method:**
    *   `dumpto(...)`: Reads from an input stream and writes Base64 encoded data to an output stream.

#### TmpFiles API
*   **File:** `src/TmpFiles.h`
*   **Purpose:** Manages the lifecycle of temporary files.
*   **Key Methods:**
    *   `add(...)`: Registers a file for tracking.
    *   `get_total_size()`: Calculates total disk usage of tracked files.
    *   `clean()`: (Private) Removes tracked files and directories.

## Data Interfaces

### Param Structure
*   **File:** `src/Param.h`
*   **Type:** Struct
*   **Purpose:** Acts as a Data Transfer Object (DTO) containing all configuration parameters (page ranges, DPI, embedding options, font settings) passed between subsystems.

### HTMLState Structures
*   **File:** `src/HTMLState.h`
*   **Types:** `FontInfo`, `HTMLTextState`, `HTMLLineState`, `HTMLClipState`
*   **Purpose:** Data structures holding the state for HTML text, lines, and clipping during conversion.

## Error Handling
*   **CLI Parsing:** `ArgParser` throws `std::string` exceptions for missing or invalid arguments.
*   **System Calls:** `pdf2htmlEX.cc` checks `errno` for system call failures (e.g., directory creation) and exits with `EXIT_FAILURE`.
*   **Runtime Checks:** `StringFormatter` uses assertions to ensure buffer integrity. `DrawingTracer` checks opacity levels to skip operations. `HTMLTextLine` prints warnings to `cerr` for potential bugs (text without style state).