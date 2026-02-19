# Data Models Documentation

## Module Overview
The `src/util` module provides core utility data structures and global configuration mappings for the application. It does not define database schemas or entity relationships in the traditional sense (e.g., SQL tables). Instead, it focuses on in-memory data structures used for font processing, text encoding, mathematical transformations, and file system abstraction.

The persistence layer in this module is strictly file-system based (handling fonts, paths, and temporary directories), with no database migrations or ORM schemas detected in the provided analysis data.

---

## Data Structures

### `FFWVersionInfo`
*   **Type:** `struct`
*   **Defined In:** `src/util/ffw.h`, `src/util/ffw.c`
*   **Description:** Instantiated to hold version information for the FontForge library wrapper.
*   **Purpose:** Encapsulates the version date information retrieved from the underlying FontForge library.
*   **Fields:** Not explicitly listed in the provided analysis data.

### `EmbedStringEntry`
*   **Type:** `struct`
*   **Defined In:** `src/util/const.h`
*   **Description:** Defines the strategy for embedding a resource (such as a font or image) into the HTML output.
*   **Purpose:** Distinguishes between embedding resources as base64 data URIs versus linking them externally.
*   **Fields:** Not explicitly listed in the provided analysis data.

---

## Global Configuration Data

The module utilizes several global maps and arrays to centralize configuration and lookup logic.

### `ID_MATRIX`
*   **Type:** `array` (6-element double)
*   **Defined In:** `src/util/const.cc`
*   **Description:** Represents the identity matrix for affine transformations.
*   **Values:** `[1.0, 0.0, 0.0, 1.0, 0.0, 0.0]`

### `GB_ENCODED_FONT_NAME_MAP`
*   **Type:** `std::map`
*   **Defined In:** `src/util/const.cc`
*   **Description:** Maps GB encoded font name byte strings to standard font names.
*   **Example Mapping:** `'\\xCB\\xCE\\xCC\\xE5'` maps to `'SimSun'`.
*   **Purpose:** Ensures correct font identification by translating encoded byte sequences into standard names (e.g., SimSun, SimHei).

### `EMBED_STRING_MAP`
*   **Type:** `std::map`
*   **Defined In:** `src/util/const.cc`
*   **Description:** Maps file extensions to `EmbedStringEntry` structures.
*   **Keys:** `.css`, `.js`, `.png`
*   **Purpose:** Determines how specific resource types should be embedded into the HTML output based on configuration parameters.

### `FORMAT_MIME_TYPE_MAP`
*   **Type:** `std::map`
*   **Defined In:** `src/util/const.cc`
*   **Description:** Maps file format extensions to their corresponding MIME type strings.
*   **Keys:** `eot`, `jpg`, `otf`, `png`, `svg`, `ttf`, `woff`.
*   **Purpose:** Provides correct MIME type declarations for embedded or linked resources.

---

## Data Validation Rules

The module enforces specific validation logic on data inputs, primarily focused on character encoding and file naming.

### Unicode Validation
*   **Rule:** Characters in the range `0x00-0x1F`, `0x7F-0xA0`, and specific control characters (like `0xAD`, `0x200B-0x200F`) are considered illegal for HTML output.
*   **Implementation:** Validated by the `is_illegal_unicode` function in `src/util/unicode.h`.
*   **Private Mapping Constraints:**
    *   Private mapping must start at `0xE600` to avoid the mobile Safari emoji range (`0xE000-0xE5FF`).
    *   If private mapping exceeds `0xF65F`, it shifts to Supplementary Private Use Area-A (`0xF0000`).
    *   If mapping exceeds `0xFFFFD`, it shifts to Supplementary Private Use Area-B (`0x100000`).

### Filename Sanitization
*   **Rule:** Filenames are considered modified (sanitized) if they contain a valid format specifier (percent sign followed by digits and 'd').
*   **Implementation:** Enforced by `sanitize_filename` in `src/util/path.h`.
*   **Constraint:** Directory creation must fail if the path exists but is not a directory.

### Floating Point Comparison
*   **Rule:** Floating point values with an absolute value less than or equal to `EPS` (epsilon) are treated as zero.
*   **Rule:** Two floating point numbers are considered equal if the absolute difference is less than or equal to `EPS`.
*   **Implementation:** Defined in `src/util/math.h`.

### Font Metric Constraints
*   **Rule:** If a loaded font is a composite (CID) font, the `cidmaster`'s ascent and descent must match the first subfont's values.
*   **Implementation:** Enforced in `ffw_load_font` within `src/util/ffw.c`.

---

## Entity Relationships

*   **Status:** Not applicable.
*   **Reasoning:** The provided analysis data does not contain entity definitions with relationships (e.g., `belongs_to`, `has_many`). The data structures identified are primarily standalone structs (`FFWVersionInfo`, `EmbedStringEntry`) or lookup maps, rather than relational entities.

---

## Persistence

*   **Database Schemas:** None detected in the provided code.
*   **Data Migration Patterns:** None detected.
*   **Caching Strategies:** None detected.
*   **File System Persistence:**
    *   The module handles file persistence for fonts (loading/saving via FontForge wrapper in `src/util/ffw.c`).
    *   It manages temporary directory creation and path manipulation (`src/util/path.cc`, `src/util/mingw.cc`).
    *   No database connection or ORM logic is present in the analyzed files.