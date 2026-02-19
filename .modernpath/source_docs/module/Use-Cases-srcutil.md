# Use Cases Documentation: `src/util`

## Overview
The `src/util` module provides the foundational utility workflows required for the core business process of converting PDF documents to HTML. It encapsulates critical logic for font manipulation, character encoding validation, file system management, mathematical transformations, and system-level error handling.

**Note:** This module acts as a backend utility library. The "User" in these use cases refers to the consuming application or system processes invoking these functions to execute the conversion pipeline.

## User Roles and Personas
*   **Status:** Not explicitly defined in code.
*   **Note:** The provided file analysis data does not contain authentication, authorization, or role-based access control logic. The module is designed for system-level consumption rather than interactive end-user roles.

## Key User Journeys and Workflows

### 1. Font Processing and Sanitization Workflow
This workflow ensures that fonts extracted from PDFs are safe, correctly encoded, and compatible with web browsers.

*   **Initialize Font Environment**
    *   **Function:** `ffw_init`
    *   **Process:** Initializes the FontForge library and configures error logging (suppressing UI warnings if not in debug mode).
*   **Load Font**
    *   **Function:** `ffw_load_font`
    *   **Process:** Loads a font from disk.
    *   **Validation:** If the font is a composite (CID) font, the system verifies that the `cidmaster`'s ascent and descent match the first subfont's values.
*   **Sanitize Font for Web**
    *   **Function:** `ffw_prepare_font`
    *   **Process:**
        *   Removes all kerning pairs and vertical kerning pairs.
        *   Removes all Alternate Unicodes to prevent conflicts during re-encoding.
        *   Wipes the font name (sets to empty string) to prevent browser rejection of malformed names.
*   **Re-encoding**
    *   **Functions:** `ffw_reencode_unicode_full`, `ffw_reencode_raw`, `ffw_reencode_raw2`, `ffw_cidflatten`
    *   **Process:** Converts the font encoding to Unicode Full, applies custom integer mappings, or applies glyph name mappings. Flattens CID fonts if necessary.
*   **Save and Close**
    *   **Functions:** `ffw_save`, `ffw_close`
    *   **Process:** Generates and saves the font script, then closes the font view.

### 2. Character Encoding and Unicode Mapping Workflow
This workflow transforms raw PDF character codes into safe, valid Unicode strings for HTML output, specifically addressing rendering quirks in browsers like mobile Safari.

*   **Validate Unicode**
    *   **Function:** `is_illegal_unicode`
    *   **Process:** Checks if a character falls into illegal ranges (e.g., 0x00-0x1F, 0x7F-0xA0, specific control characters) defined by WebKit/Mozilla specs.
*   **Map to Private Use Area (PUA)**
    *   **Function:** `map_to_private`
    *   **Process:**
        *   Maps characters to the Unicode Private Use Area.
        *   **Constraint:** Starts at 0xE600 to avoid the mobile Safari emoji range (0xE000-0xE5FF).
        *   **Overflow Handling:** If mapping exceeds 0xF65F, shifts to Supplementary Private Use Area-A (0xF0000). If it exceeds 0xFFFFD, shifts to Supplementary Private Use Area-B (0x100000).
*   **Extract and Check Unicode**
    *   **Functions:** `unicode_from_font`, `check_unicode`
    *   **Process:** Attempts to extract Unicode values from font data. If a sequence represents a ligature, it attempts restoration; otherwise, it falls back to private mapping.

### 3. Text Serialization and Escaping Workflow
This workflow ensures that text data is safely serialized into different output formats (HTML, JSON, Attributes) to prevent injection attacks and syntax errors.

*   **Write HTML Content**
    *   **Function:** `writeUnicodes`
    *   **Process:** Converts Unicode arrays/vectors to UTF-8 output streams.
    *   **Escaping:** Escapes specific characters (`&`, `"`, `'`, `<`, `>`) into HTML entities.
*   **Write HTML Attributes**
    *   **Function:** `writeAttribute`
    *   **Process:** Escapes strings for use within HTML tag attributes.
    *   **Security Rule:** Escapes backticks (`` ` ``) to mitigate Internet Explorer specific vulnerabilities (html5sec.org/#59).
*   **Write JSON Data**
    *   **Function:** `writeJSON`
    *   **Process:** Escapes control characters and specific quotes using backslashes for valid JSON syntax.

### 4. File System and Path Management Workflow
This workflow handles the creation of output directories and the sanitization of filenames to ensure file system validity and security.

*   **Create Output Structure**
    *   **Function:** `create_directories`
    *   **Process:** Recursively creates a directory structure.
    *   **Error Handling:** Throws an error if the path exists but is not a directory.
*   **Sanitize Filenames**
    *   **Function:** `sanitize_filename`
    *   **Process:** Escapes `%` characters in filenames unless they form a valid integer format specifier (e.g., `%d`), preventing format string injection issues.
*   **Identify Resources**
    *   **Functions:** `is_truetype_suffix`, `get_suffix`
    *   **Process:** Identifies if files are TrueType fonts (`.ttf`, `.ttc`, `.otf`) or extracts file extensions for MIME type lookup.

### 5. Crash Diagnostics Workflow
This workflow manages application stability by handling fatal signals and providing diagnostic context.

*   **Setup Diagnostics**
    *   **Function:** `setupSignalHandler`
    *   **Process:** Registers handlers for fatal signals (SIGSEGV, SIGILL, etc.).
    *   **Requirement:** Must be invoked after command-line argument parsing to ensure accurate diagnostic reporting.
*   **Context Tracking**
    *   **Functions:** `ffwSetAction`, `ffwClearAction`
    *   **Process:** Sets/Clears a global action string (e.g., "Loading Font") so that if a crash occurs, the error message includes the specific operation being performed.

## Business Process Flows

### Font Name Standardization
*   **Source:** `const.cc` / `const.h`
*   **Flow:** The system utilizes a predefined map (`GB_ENCODED_FONT_NAME_MAP`) to translate GB encoded font byte strings into standard font names (e.g., mapping specific byte sequences to 'SimSun' or 'SimHei').

### Resource Embedding Strategy
*   **Source:** `const.cc` / `const.h`
*   **Flow:** The system consults the `EMBED_STRING_MAP` to determine how to handle resources (CSS, JS, PNG). It decides whether to embed them as Base64 data URIs or link them externally based on configuration parameters.

### CSS Geometry Correction
*   **Source:** `misc.cc` / `misc.h`
*   **Flow:** When rendering borders, the system applies `css_fix_rectangle_border_width`.
*   **Rule:** PDF borders are centered on the edge, while HTML borders are outside. The function adjusts dimensions: if a rectangle width/height is less than the border width, the dimension is set to 0, and the border width is expanded to cover the center.

## Edge Cases and Error Handling

*   **Private Mapping Exhaustion:** If all Private Use Area ranges are exhausted during character mapping, the system outputs a warning to `stderr` (from `unicode.cc`).
*   **Directory Existence Conflicts:** In `path.cc`, directory creation logic explicitly checks `errno` for `EEXIST` but fails if the existing path is not a directory.
*   **CID Font Metric Mismatches:** In `ffw.c`, loading a CID font triggers a check where metric mismatches between the master and subfont result in a failure/exit condition via the internal `err` function.
*   **Invalid UTF-8 Conversion:** In `encoding.cc`, the `mapUTF8` function returns 0 if the buffer is too small or the character is invalid, preventing buffer overflows.

## Common Usage Patterns

*   **Platform Abstraction:** The module heavily uses conditional compilation (evident in `mingw.cc` and `SignalHandler.cc`) to provide POSIX functionality on Windows (e.g., `mkdtemp`, signal handling).
*   **Stream Processing:** Text output operations consistently use C++ `std::ostream` references, allowing for flexible output targets (files, memory buffers).
*   **Configuration Lookup:** Business rules regarding MIME types and embedding strategies are centralized in static maps (`FORMAT_MIME_TYPE_MAP`, `EMBED_STRING_MAP`) rather than hardcoded in logic functions.