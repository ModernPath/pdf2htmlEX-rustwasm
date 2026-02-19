# Code Structure Documentation: `src`

## Module Overview
The `src` module serves as the core subsystem for a PDF to HTML conversion application. It is composed of 24 files written in C and C++, totaling 3,827 lines of code. The module is responsible for the entire conversion pipeline, ranging from command-line argument parsing and PDF preprocessing (metadata extraction) to the geometric analysis of drawing operations and the final generation of optimized HTML/CSS markup.

## Directory Structure
The module is organized as a flat directory structure within `src/`. The provided analysis does not indicate any subdirectories within `src` itself, though internal dependencies reference a sibling `util` directory.

**Key Files:**
*   **Entry Point:** `pdf2htmlEX.cc`
*   **Configuration:** `ArgParser.h/cc`, `Param.h`
*   **Analysis & Geometry:** `Preprocessor.h/cc`, `DrawingTracer.h/cc`, `CoveredTextDetector.h/cc`
*   **HTML Generation:** `HTMLTextPage.h/cc`, `HTMLTextLine.h/cc`, `StateManager.h`, `HTMLState.h`
*   **Utilities:** `Color.h/cc`, `Base64Stream.h/cc`, `StringFormatter.h/cc`, `TmpFiles.h/cc`

## Key Components and Responsibilities

### 1. Application Orchestration
*   **`pdf2htmlEX.cc`**: The main entry point. It initializes global parameters, parses command-line arguments via `ArgParser`, prepares temporary directories, and invokes the `HTMLRenderer` to perform the conversion.

### 2. Configuration & Utilities
*   **`Param` (struct)**: A Data Transfer Object (DTO) that aggregates all configuration settings (DPI, page ranges, embedding options, font settings) used throughout the application.
*   **`ArgParser`**: A wrapper around the standard `getopt` library. It provides a fluent interface and template-based strategy for registering and parsing command-line arguments.
*   **`TmpFiles`**: An RAII-based utility that tracks temporary files and directories, ensuring they are cleaned up upon object destruction based on configuration flags.
*   **`StringFormatter`**: A utility class that manages a reusable buffer for efficient printf-style string formatting, utilizing a guard pointer for buffer access.
*   **`Base64Stream`**: A stream decorator that encodes binary input streams into Base64 format, likely used for embedding assets.

### 3. PDF Analysis & Geometry
*   **`Preprocessor`**: Extends Poppler's `OutputDev` to perform a preliminary scan of the PDF document. It collects metadata such as font usage (character codes), link destinations, and maximum page dimensions before the main rendering pass.
*   **`DrawingTracer`**: An Observer/Command pattern implementation that intercepts drawing operations (paths, strokes, fills) from the PDF rendering pipeline. It replays these operations onto a Cairo recording surface to calculate bounding boxes for occlusion detection.
*   **`CoveredTextDetector`**: Tracks character bounding boxes and tests them against non-character graphical elements (strokes, fills) to determine text visibility/occlusion.

### 4. HTML Generation Engine
*   **`StateManager`**: A template-based system implementing the Flyweight pattern. It manages CSS state generation (fonts, colors, transforms) by assigning unique IDs to distinct values, ensuring duplicate style values are not repeated in the output.
    *   *Specializations:* `FontSizeManager`, `ColorManager`, `TransformMatrixManager`, etc.
*   **`HTMLState`**: Defines data structures (`FontInfo`, `HTMLTextState`, `HTMLLineState`, `HTMLClipState`) representing the state of HTML output generation.
*   **`HTMLTextLine`**: Manages a single line of text. It handles sequences of characters, offsets (whitespace), and style changes. It optimizes the structure by merging styles and handles the serialization of the line into HTML/CSS.
*   **`HTMLTextPage`**: Aggregates multiple `HTMLTextLine` objects. It manages clipping regions and orchestrates the dumping of text and CSS for an entire page.

## Code Patterns and Architectural Decisions

*   **Flyweight Pattern**: Used extensively in `StateManager` to minimize CSS output size by sharing intrinsic state (style values) across the document.
*   **Observer Pattern**: `DrawingTracer` and `Preprocessor` hook into the Poppler rendering pipeline (extending `OutputDev`) to intercept events without modifying the core library logic.
*   **RAII (Resource Acquisition Is Initialization)**: Used in `TmpFiles` for file lifecycle management and `StringFormatter` for buffer management.
*   **Template Method**: `Preprocessor` and `DrawingTracer` override specific virtual methods from `OutputDev` to define custom behavior at specific steps of the rendering process.
*   **State Machine**: `HTMLTextLine` utilizes a state machine approach to manage text rendering states and style transitions.
*   **Value Objects**: `Color` is implemented as a Value Object, wrapping external PDF graphics types (`GfxRGB`) to standardize color handling.

## Dependency Graph

The following relationships are derived from the internal and external dependencies listed in the file analysis.

```mermaid
graph TD
    %% Entry Point
    Entry[pdf2htmlEX.cc] --> ArgParser[ArgParser]
    Entry --> Param[Param.h]
    Entry --> HTMLRenderer[HTMLRenderer] %% External reference

    %% Configuration
    ArgParser --> StdLib[Standard C++ Lib]
    TmpFiles[TmpFiles] --> Param
    TmpFiles --> StdLib

    %% Preprocessing
    Preprocessor[Preprocessor] --> Param
    Preprocessor --> Poppler[Poppler OutputDev]

    %% Rendering/Tracing
    DrawingTracer[DrawingTracer] --> Param
    DrawingTracer --> Cairo[Cairo]
    DrawingTracer --> UtilMath[util/math.h]

    CoveredTextDetector[CoveredTextDetector] --> Param
    CoveredTextDetector --> UtilMath

    %% HTML Generation
    HTMLTextPage[HTMLTextPage] --> HTMLTextLine[HTMLTextLine]
    HTMLTextPage --> StateManager[StateManager]
    HTMLTextPage --> HTMLState[HTMLState]

    HTMLTextLine --> StateManager
    HTMLTextLine --> HTMLState
    HTMLTextLine --> UtilEncoding[util/encoding.h]

    StateManager --> Color[Color.h]
    StateManager --> UtilCSS[util/css_const.h]
    StateManager --> StdLib

    %% Utilities
    Color --> StdLib
    Base64Stream[Base64Stream] --> StdLib
    StringFormatter[StringFormatter] --> StdLib
```

## External Dependencies
The module relies on the following external libraries and systems, as evidenced by import statements:

*   **Poppler PDF Library**: `GfxState.h`, `GfxFont.h`, `PDFDoc.h`, `OutputDev.h`, `Object.h`, `GlobalParams.h`. Used for PDF parsing and rendering primitives.
*   **Cairo Graphics Library**: `cairo.h`. Used for recording drawing operations and geometric calculations.
*   **Standard C++ Library**: `iostream`, `vector`, `string`, `map`, `unordered_map`, `algorithm`, `memory`, `functional`.
*   **Standard C Library**: `cstdio`, `cstdlib`, `cstring`, `ctime`, `cerrno`, `getopt.h`, `sys/stat.h`, `unistd.h`.
*   **Internal Sibling Modules**:
    *   `util/`: Referenced for `encoding.h`, `math.h`, `css_const.h`, `misc.h`, `const.h`, `mingw.h`.

## Entry Points
*   **`pdf2htmlEX.cc`**: This is the explicit entry point containing the `main` function logic. It initializes the application, parses arguments, and invokes the renderer.

## Build Configuration
*   **Status**: Not determined from available code. (No `Makefile`, `CMakeLists.txt`, or similar build scripts were included in the file analysis data).