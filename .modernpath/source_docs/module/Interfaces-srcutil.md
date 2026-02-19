# Interfaces Documentation: `src/util`

## Overview
The `src/util` module acts as a **Core Utility Library** written in C and C++. It does not expose REST/GraphQL endpoints, WebSocket protocols, or network-based APIs. Instead, it provides a set of **C/C++ library interfaces** and **integration wrappers** used by other subsystems within the application.

Its primary role is to abstract external dependencies (FontForge, Poppler, OS signals) and provide low-level utilities for file system manipulation, mathematical transformations, and text encoding.

## External Integrations
This module integrates directly with the following external libraries and system interfaces. No HTTP-based external service integrations were detected.

### External Library Wrappers
*   **FontForge Library**
    *   **Integration Point:** `src/util/ffw.c`, `src/util/ffw.h`
    *   **Protocol:** C-linkage wrapper (`extern "C"`) around FontForge's internal API.
    *   **Purpose:** Manages font loading, saving, re-encoding, and metric adjustments.
    *   **Key Dependencies:** `fontforge.h`, `baseviews.h`, `fontforge/encoding.h`, `fontforge/savefont.h`, and others.

*   **Poppler Library**
    *   **Integration Point:** `src/util/unicode.h`, `src/util/misc.cc`
    *   **Protocol:** Direct usage of Poppler types (`GfxFont`, `GfxRGB`, `GfxState`).
    *   **Purpose:** Extracts font information and handles color definitions from PDF structures.

### System-Level Integrations
*   **Operating System Signals**
    *   **Integration Point:** `src/util/SignalHandler.cc`
    *   **Protocol:** POSIX `sigaction` and Windows signal handling.
    *   **Purpose:** Intercepts fatal signals (e.g., `SIGSEGV`, `SIGFPE`, `SIGILL`) to log diagnostic information before process termination.

*   **File System (POSIX/Windows)**
    *   **Integration Point:** `src/util/path.cc`, `src/util/mingw.cc`
    *   **Protocol:** System calls (`mkdir`, `stat`, `mkdtemp`).
    *   **Purpose:** Provides cross-platform directory creation and path manipulation.

---

## Core Library APIs

### 1. FontForge Wrapper API
Located in `src/util/ffw.h` and implemented in `src/util/ffw.c`, this API exposes a C-compatible interface for font processing.

**Initialization & Lifecycle**
*   `void ffw_init()`: Initializes the FontForge library, sets default encodings, and configures error logging.
*   `void ffw_finalize()`: Cleans up library state and frees memory.
*   `const FFWVersionInfo* ffw_get_version_info()`: Retrieves version metadata.

**Font Management**
*   `void ffw_new_font()`: Creates a new empty font object.
*   `void ffw_load_font(const char* filename)`: Loads a font from disk.
*   `void ffw_close()`: Closes the current font view.
*   `void ffw_save(const char* filename)`: Saves the font script to a file.

**Font Manipulation**
*   `void ffw_prepare_font()`: Sanitizes the font (removes kerning, alt unicodes, resets names).
*   `void ffw_reencode_glyph_order()`: Re-encodes to original glyph order.
*   `void ffw_reencode_unicode_full()`: Re-encodes to Unicode Full.
*   `void ffw_reencode_raw(int* map)`: Re-encodes using an integer mapping array.
*   `void ffw_reencode_raw2(const char** names)`: Re-encodes using a glyph name mapping array.
*   `void ffw_cidflatten()`: Flattens CID fonts.
*   `void ffw_add_empty_char(int unicode, int width)`: Adds an empty glyph.
*   `void ffw_import_svg_glyph(const char* filename)`: Imports an SVG file as a glyph.

**Metrics & Hinting**
*   `int ffw_get_em_size()`: Returns the calculated EM size.
*   `void ffw_get_metric(int* ascent, int* descent)`: Retrieves font metrics.
*   `void ffw_set_metric(int ascent, int descent)`: Sets font metrics.
*   `void ffw_fix_metric()`: Automatically fixes metrics based on shape.
*   `void ffw_set_widths(int* widths)`: Sets glyph widths.
*   `void ffw_auto_hint()`: Applies auto-hinting.
*   `void ffw_override_fstype()`: Overrides embedding restrictions.

### 2. Text Encoding & Serialization API
Located in `src/util/encoding.h` and `src/util/encoding.cc`, this interface handles the conversion and escaping of text for output streams.

**Stream Writers**
*   `void writeUnicodes(std::ostream& out, const Unicode* u, int uLen)`: Writes a C-style Unicode array to the stream with HTML escaping.
*   `void writeUnicodes(std::ostream& out, const std::vector<Unicode>& u)`: Writes a `std::vector` of Unicode to the stream with HTML escaping.
*   `void writeJSON(std::ostream& out, const std::string& s)`: Writes a string to the stream with JSON escaping (quotes, backslashes, control chars).
*   `void writeAttribute(std::ostream& out, const std::string& s)`: Writes a string to the stream with HTML attribute escaping (includes backtick escaping for IE security).

**Low-level Encoding**
*   `int mapUTF8(unsigned int val, char* buf)`: Converts a single Unicode character to UTF-8 bytes.

### 3. System Signal Handling API
Located in `src/util/SignalHandler.h` and `src/util/SignalHandler.cc`.

*   `void setupSignalHandler(int argc, const char* argv[], const char* data_dir, const char* poppler_data_dir, const char* tmp_dir)`: Initializes the signal handler with application context.
*   `void ffwSetAction(const char* anAction)`: Sets a global action string for crash context.
*   `void ffwClearAction(const char* anAction)`: Clears the global action string.

### 4. Filesystem & Path API
Located in `src/util/path.h` and `src/util/path.cc`.

*   `void create_directories(const std::string& path)`: Recursively creates directories.
*   `bool sanitize_filename(std::string& name)`: Escapes format specifiers in filenames.
*   `bool is_truetype_suffix(const std::string& suffix)`: Checks for `.ttf`, `.ttc`, `.otf` extensions.
*   `std::string get_filename(const std::string& path)`: Extracts the filename.
*   `std::string get_suffix(const std::string& path)`: Extracts the file extension.

### 5. Unicode Utility API
Located in `src/util/unicode.h` and `src/util/unicode.cc`.

*   `bool is_illegal_unicode(Unicode u)`: Validates if a character is illegal for HTML (control chars, etc.).
*   `Unicode map_to_private(int code)`: Maps a character code to a Private Use Area (PUA) Unicode value.
*   `Unicode unicode_from_font(...)`: Extracts Unicode from font data.
*   `Unicode check_unicode(...)`: Validates and re-encodes sequences, handling ligatures.

### 6. Mathematical & Geometric API
Located in `src/util/math.h` and `src/util/math.cc`.

*   `void tm_transform(double* tm, double* x, double* y)`: Applies a transformation matrix to coordinates.
*   `void tm_multiply(double* tm_left, double* tm_right)`: Multiplies transformation matrices.
*   `void tm_transform_bbox(double* tm, double* bbox)`: Transforms a bounding box.
*   `bool bbox_intersect(double* bbox1, double* bbox2, double* result)`: Calculates bounding box intersection.

---

## Data Formats & Serialization Protocols

### Text Encoding Rules
The module enforces specific escaping rules when serializing data to streams:

*   **HTML Text:** Characters like `&`, `"`, `'`, `<`, `>` must be escaped to HTML entities.
*   **HTML Attributes:** In addition to standard HTML escaping, backticks (`` ` ``) must be escaped to mitigate Internet Explorer vulnerabilities (html5sec.org/#59).
*   **JSON:** Control characters and specific quotes must be escaped with backslashes.

### Font Encoding Strategies
*   **Private Use Area (PUA) Mapping:** Characters are mapped to specific Unicode ranges (starting at `0xE600`) to avoid conflicts with mobile Safari emoji ranges (`0xE000-0xE5FF`).
*   **Fallback Strategy:** If PUA ranges are exhausted, mapping shifts to Supplementary Private Use Area-A (`0xF0000`) or Area-B (`0x100000`).

### Configuration Data Structures
*   `EmbedStringEntry`: Defines strategies for embedding resources (Base64 vs. external linking).
*   `GB_ENCODED_FONT_NAME_MAP`: Maps GB encoded byte strings to standard font names (e.g., SimSun).
*   `FORMAT_MIME_TYPE_MAP`: Maps file extensions (eot, jpg, otf, etc.) to MIME type strings.

---

## Error Handling & Status Codes

The module utilizes several distinct error handling mechanisms depending on the subsystem:

### Process-Level Termination (FontForge Wrapper)
*   **Mechanism:** Internal `err` function.
*   **Behavior:** Prints error messages to `stderr` and immediately calls `exit()` on critical failures (e.g., memory allocation failures, invalid states).

### Exception Handling (Filesystem)
*   **Mechanism:** C++ Exceptions.
*   **Behavior:** Functions in `path.cc` throw C++ `std::string` objects on failure (e.g., "Cannot create directory").

### Crash Reporting (Signal Handler)
*   **Mechanism:** OS Signal Interception.
*   **Behavior:** Catches `SIGSEGV`, `SIGILL`, `SIGFPE`, etc. It writes diagnostic information (version, args, paths) to `stderr` using low-level I/O (to ensure safety during crashes) before forcing an exit.

### Return Value Indicators
*   **Encoding:** `mapUTF8` returns `0` if the buffer is too small or the character is invalid.
*   **Sanitization:** `sanitize_filename` returns `true` if a format specifier was found and modified.
*   **Geometry:** `bbox_intersect` returns `false` if boxes do not overlap.