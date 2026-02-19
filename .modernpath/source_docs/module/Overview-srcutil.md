# Module Overview: src/util

## Executive Summary

The `src/util` module serves as the foundational utility layer for a PDF-to-HTML conversion system. It solves the complex problem of translating static PDF document structures—specifically fonts, text encodings, and coordinate geometries—into dynamic, web-compatible formats. The module ensures that the visual fidelity of the original document is preserved in the HTML output while adhering to the constraints and security requirements of modern web browsers.

The value provided by this module lies in its ability to abstract away low-level system complexities. It handles critical cross-cutting concerns such as sanitizing fonts for web embedding, mapping Unicode characters to prevent rendering conflicts (such as emoji overlaps), and securing text output against injection attacks. By managing platform-specific differences (e.g., Windows vs. POSIX file systems) and providing robust error reporting, the module ensures the conversion process is stable, secure, and consistent across different operating environments.

## Business Purpose and Goals

Based on the explicit logic and rules defined within the code, the module is driven by the following technical and operational goals:

*   **Ensure Web Compatibility:** To sanitize and prepare fonts for web usage by removing features that browsers reject, such as malformed font names, kerning pairs, and conflicting alternate Unicode mappings.
*   **Prevent Rendering Artifacts:** To eliminate browser-specific rendering quirks by identifying and remapping illegal Unicode characters and avoiding ranges reserved for system fonts (e.g., mobile Safari emoji).
*   **Security and Validity:** To guarantee that generated HTML and JSON outputs are syntactically correct and secure by enforcing strict escaping rules for special characters and attributes.
*   **Cross-Platform Stability:** To provide a consistent execution environment across Windows and Unix-like systems by abstracting file system operations and signal handling.

## Key Capabilities and Features

The module implements specific capabilities evidenced by the code logic:

*   **Font Processing and Sanitization:**
    *   Wraps the FontForge library to load, process, and save fonts.
    *   Automatically cleans font metadata by removing vertical kerning, alternate unicodes, and resetting font names to prevent browser rejection.
    *   Supports complex font manipulations including CID flattening, metric fixing, and custom re-encoding (Unicode Full, raw mappings).
*   **Unicode and Text Management:**
    *   Validates characters against WebKit and Mozilla specifications to identify illegal sequences (e.g., control characters, bidirectional controls).
    *   Maps problematic characters to Unicode Private Use Areas (PUA), specifically avoiding the 0xE000-0xE5FF range used by mobile Safari emoji.
    *   Handles ligature expansions and extracts Unicode values directly from font data.
*   **Output Security and Encoding:**
    *   Escapes text content for HTML entities (e.g., `&`, `<`, `>`) to prevent rendering errors.
    *   Implements specific security measures for HTML attributes, such as escaping backticks to mitigate Internet Explorer vulnerabilities.
    *   Provides JSON string escaping to ensure valid data serialization.
*   **Geometric Transformation:**
    *   Performs affine transformations (matrix multiplication, point transformation) to map PDF coordinate systems to HTML/Canvas coordinates.
    *   Calculates bounding box intersections to determine layout overlaps.
    *   Adjusts CSS rectangle dimensions and border widths to account for differences between PDF (border centered) and HTML (border outside) rendering models.
*   **File System and Platform Abstraction:**
    *   Recursively creates directory structures and handles file path manipulation.
    *   Sanitizes filenames to prevent format string injection attacks.
    *   Provides Windows/MinGW compatibility layers (polyfills) for standard POSIX functions like `mkdtemp`.

## Target Audience/Users

*   **Not explicitly documented** in the codebase. The code contains technical constraints regarding browser compatibility (WebKit, Mozilla, Mobile Safari) but does not define specific user personas or business roles.

## Business Domain Context

The module operates within the **Document Rendering and Digital Publishing** domain. It specifically addresses the sub-domain of **Format Migration**, converting fixed-layout PDF documents into reflowable or interactive HTML formats.

The code explicitly references interactions with:
*   **Typography and Font Engineering:** Managing glyph orders, CID fonts, and TrueType/OpenType specifications.
*   **Web Standards:** Adhering to HTML5, CSS3, and JSON syntax rules.
*   **Browser Compatibility:** Addressing specific behaviors in rendering engines like WebKit and Mozilla.

## High-Level Architecture

The module is organized into functional components that handle specific aspects of the conversion pipeline:

```mermaid
graph TD
    subgraph "Core Utilities (src/util)"
        A[Font Processing Wrapper] --> B[Signal Handling & Diagnostics]
        C[Text & Encoding Engine] --> D[Geometric Math Engine]
        E[File System & Platform Layer] --> F[Configuration & Constants]
    end

    subgraph "External Dependencies"
        G[FontForge Library]
        H[Poppler / PDF Library]
        I[OS Layer (POSIX/Windows)]
    end

    A -.-> G
    C -.-> H
    E -.-> I
```

*   **Font Processing Wrapper:** Interfaces with the FontForge library to handle font lifecycle (init, load, prepare, save).
*   **Text & Encoding Engine:** Manages Unicode validation, mapping strategies, and output escaping (HTML/JSON).
*   **Geometric Math Engine:** Provides the mathematical foundation for coordinate transformations and layout calculations.
*   **File System & Platform Layer:** Abstracts OS-specific file operations and ensures cross-platform directory creation.
*   **Signal Handling & Diagnostics:** Captures fatal process errors to provide diagnostic logging for crashes.
*   **Configuration & Constants:** Centralizes global settings, including MIME type maps and embedding strategies.

## Technology Stack Summary

*   **Languages:** C, C++
*   **Core Libraries:**
    *   **FontForge:** Used for font manipulation, sanitization, and re-encoding.
    *   **Poppler:** Inferred dependency for PDF graphics state (`GfxState`, `GfxFont`) and global parameters.
    *   **Cairo:** Referenced in signal handling contexts.
*   **System Interfaces:** POSIX standard library, Windows API (via MinGW wrappers).

## Key Metrics and Scale

*   **Total Files:** 19
*   **Total Lines of Code:** 1,995
*   **Complexity Assessment:** The module ranges from low to high complexity, with the FontForge wrapper (`ffw.c`) identified as the most complex component due to the intricacies of font manipulation and library state management.