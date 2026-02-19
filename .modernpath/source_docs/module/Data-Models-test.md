# Data Models Documentation: `test`

## Overview
The `test` module serves as the verification suite for the HTML Rendering Engine. From a data perspective, this module does not define traditional database schemas or persistent entity relationships. Instead, it relies on **configuration data structures** to control test execution environments and **file-based structural schemas** (HTML/CSS) that define the expected output format of the rendering engine.

The data models in this module are primarily concerned with:
1.  **Test Configuration:** Matrices and argument lists defining browser environments and tool parameters.
2.  **Output Artifacts:** The structural schema of generated HTML files, including specific CSS classes, IDs, and embedded font data used for visual regression testing.
3.  **File System Conventions:** Naming conventions and file generation rules for converted outputs.

---

## Configuration Data Structures

### `DEFAULT_PDF2HTMLEX_ARGS`
*   **Type:** `list`
*   **Location:** `test/browser_tests.py`
*   **Description:** A data structure holding the default command-line arguments passed to the `pdf2htmlEX` tool during testing. This ensures consistent parameters (such as fit-width and last-page limits) across test cases.

### `BROWSER_MATRIX`
*   **Type:** `list` of `tuples`
*   **Location:** `test/test_remote_browser.py`
*   **Description:** A configuration matrix defining the target environments for cross-browser testing. Each tuple specifies the Operating System, Browser Name, and Browser Version.
*   **Usage:** Used to dynamically generate test classes for each specific browser/OS combination.

### `SAUCE_OPTIONS`
*   **Type:** `map` (dictionary)
*   **Location:** `test/test_remote_browser.py`
*   **Description:** A configuration dictionary for Sauce Labs session options. It contains key-value pairs that control remote browser session behavior, such as video recording settings.

---

## HTML/CSS Structural Schemas (Test Artifacts)

The module utilizes a consistent HTML/CSS schema across its generated test fixtures (e.g., `basic_text.html`, `fontfile3_opentype.html`). These structures define the expected DOM layout for the rendering engine's output.

### Core DOM Identifiers
*   **`#sidebar`**: CSS selector for the sidebar element.
*   **`#page-container`**: CSS selector for the main page container.
    *   **Constraint:** Defined in `test/fancy.min.css`, the `overflow` property for this ID is set to `hidden` to ensure consistent viewport dimensions during screenshot capture.

### Structural CSS Classes
The following classes are used consistently across test fixtures to define the layout hierarchy:
*   **`.pf`**: Class for page frames (defines relative positioning and white background).
*   **`.pc`**: Class for page content (uses absolute positioning and transform origins).
*   **`.bf`**: Class for background frames (covers full page area).
*   **`.bi`**: Class for background images (positioned absolutely).
*   **`.c`**: Class for content clipping containers.
*   **`.t`**: Class for text layers (uses absolute positioning and pre-formatted whitespace).

### Font Family Classes
*   **`.ff0`**: Font family class for sans-serif fallback.
*   **`.ff1`**: Font family class for the primary embedded font.
*   **`.ff2`**: Font family class for secondary embedded fonts (observed in `geneve_1564.html` and `svg_background_with_page_rotation_issue402.html`).

### Embedded Data Patterns
*   **Base64 Fonts:** Test fixtures embed font data directly as Data URIs (e.g., `font-woff` or `opentype`) to ensure rendering consistency without external dependencies.
*   **Inline CSS:** Styling is embedded directly within the HTML files to isolate test cases from external stylesheet variations.

---

## Data Validation & Constraints

### File Naming Conventions
The module enforces specific rules regarding the persistence and naming of generated HTML files, as defined in `test/test_output.py`:

1.  **Default Naming:** When converting a PDF without specifying an output name, the resulting HTML file **must** share the same base name as the input PDF.
2.  **Split Pages Logic:** When using the `--split-pages` option:
    *   A main HTML file **must** be generated alongside individual page files.
    *   Filename formatters **must** support `%d` for page numbers.
    *   Filename formatters **must** support padding specifiers like `%03d`.

### Test Data Integrity
*   **Pre-compiled Precedence:** As defined in `test/browser_tests.py`, if a pre-compiled HTML file exists in the `PREDIR`, it **must** be used instead of running the live conversion tool. This ensures tests run against known data states.
*   **Visual Fidelity:** Visual differences between the rendered output and the reference image are treated as data validation failures. The system calculates the percentage of differing pixels to determine if the data (visual output) is valid.

---

## Persistence Strategy

The module does not utilize a database. Persistence is handled entirely through the **File System**:

1.  **Input Data:** PDF files located in test directories.
2.  **Generated Artifacts:**
    *   HTML files (converted from PDFs).
    *   PNG files (screenshots captured by Selenium).
3.  **Reference Data:**
    *   Pre-compiled HTML files used for regression testing.
    *   Reference images used for pixel-diff comparison.
4.  **Configuration:**
    *   Environment variables (e.g., `USERNAME`, `ACCESS_KEY` for Sauce Labs) are read at runtime but not persisted by the module itself.

---

## Entity Relationships (Code Level)

While not database entities, the following class inheritance relationships define the data handling capabilities of the test suite:

*   **`test_remote_browser_base`** (in `test/test_remote_browser.py`)
    *   *Inherits from:* `BrowserTests` (in `test/browser_tests.py`)
    *   *Role:* Extends base test data structures to support remote WebDriver configuration and Sauce Labs reporting.

*   **`test_local_browser`** (in `test/test_local_browser.py`)
    *   *Inherits from:* `BrowserTests` (in `test/browser_tests.py`)
    *   *Role:* Extends base test data structures to support local Firefox WebDriver execution.