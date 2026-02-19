# Code Structure Documentation: `test`

## Overview
The `test` module is a standard Python-based test suite designed to validate the output of a PDF-to-HTML conversion engine (specifically `pdf2htmlEX`). The architecture focuses on **Visual Regression Testing** and **File Generation Verification**. It utilizes a Template Method pattern to separate the abstract test workflow from specific browser implementations (Local vs. Remote/Sauce Labs).

The module is organized into Python test runners, static HTML test fixtures, CSS styling for testing, and legacy batch scripts.

## Directory Structure

```text
test/
├── README.md                      # Documentation for test suite procedures
├── fancy.min.css                  # CSS constraints for visual testing (hides overflow)
├── browser_tests.py               # Base class and core logic for visual regression
├── test_local_browser.py          # Concrete implementation for local Firefox testing
├── test_remote_browser.py         # Concrete implementation for Sauce Labs remote testing
├── test_output.py                 # Unit tests for file generation and naming logic
├── old/
│   └── test.py                    # Legacy batch processing script
└── browser_tests/                 # Directory containing HTML test fixtures
    ├── basic_text/
    │   └── basic_text.html
    ├── fontfile3_opentype/
    │   └── fontfile3_opentype.html
    ├── geneve_1564/
    │   └── geneve_1564.html
    ├── invalid_unicode_issue477/
    │   └── invalid_unicode_issue477.html
    ├── svg_background_with_page_rotation_issue402/
    │   └── svg_background_with_page_rotation_issue402.html
    ├── text_visibility/
    │   └── text_visibility.html
    └── with_form/
        └── with_form.html
```

## Key Components

### 1. Core Test Logic (`browser_tests.py`)
This file defines the foundational architecture for visual testing.

*   **Class: `BrowserTests`**
    *   **Type:** Base Class (Abstract in behavior)
    *   **Responsibility:** Implements the workflow for PDF conversion, image generation, and visual comparison.
    *   **Key Methods:**
        *   `run_test_case()`: Orchestrates the end-to-end process (PDF -> HTML -> Screenshot -> Compare).
        *   `test_fail()`: Negative test case ensuring the failure detection mechanism works.
        *   `test_basic_text()`: Validates basic text rendering.
        *   `test_svg_background_with_page_rotation_issue402()`: Regression test for specific rendering issues.
    *   **Data Structures:**
        *   `DEFAULT_PDF2HTMLEX_ARGS`: List of default command-line arguments for the conversion tool.

### 2. Browser Implementations
The module extends `BrowserTests` to support different execution environments.

*   **File: `test_local_browser.py`**
    *   **Class: `test_local_browser`**
        *   **Inheritance:** Inherits from `BrowserTests`.
        *   **Responsibility:** Manages a local Firefox WebDriver instance.
        *   **Key Methods:**
            *   `setUpClass()`: Initializes local driver and enforces window size (`BROWSER_WIDTH`, `BROWSER_HEIGHT`).
            *   `generate_image()`: Loads HTML via `file://` protocol and captures screenshots.
            *   `tearDownClass()`: Cleans up the browser session.

*   **File: `test_remote_browser.py`**
    *   **Class: `test_remote_browser_base`**
        *   **Inheritance:** Inherits from `BrowserTests`.
        *   **Responsibility:** Manages Sauce Labs remote browser sessions and reporting.
        *   **Key Methods:**
            *   `generate_classes()`: Dynamically creates test classes for each OS/Browser/Version combination in `BROWSER_MATRIX`.
            *   `setUpClass()`: Configures remote WebDriver capabilities.
            *   `tearDown()`: Reports pass/fail status and build metadata to Sauce Labs.
            *   `generate_image()`: Navigates to URL and captures remote screenshots.
    *   **Data Structures:**
        *   `BROWSER_MATRIX`: List of tuples defining target OS, browser, and versions.
        *   `SAUCE_OPTIONS`: Configuration dictionary for session options (e.g., video recording).

### 3. Output Verification (`test_output.py`)
This file handles functional testing of the conversion tool's file I/O logic, independent of visual rendering.

*   **Class: `test_output`**
    *   **Responsibility:** Verifies correct file generation and naming conventions.
    *   **Key Methods:**
        *   `run_test_case()`: Executes conversion and asserts expected output files exist.
        *   `test_generate_single_html_default_name_single_page_pdf()`: Checks default naming behavior.
        *   `test_generate_split_pages_specify_name_formatter_with_padded_zeros_multiple_pages()`: Validates split-page logic with zero-padding.
        *   `test_issue501()`: Regression test for specific split-page/CSS embedding issues.

### 4. Legacy Batch Processing (`old/test.py`)
*   **Script:** `test.py`
*   **Responsibility:** Iterates through a directory of PDFs, converts them using `os.system`, and generates an index HTML file.
*   **Error Handling:** Halts execution immediately (`sys.exit(-1)`) if any conversion fails.

## Test Fixtures (HTML/CSS)

The `browser_tests/` directory contains static HTML files serving as **Golden Master** fixtures or regression test cases.

*   **Common Structure (HTML files):**
    *   **CSS Classes:** Standardized classes for layout:
        *   Containers: `#sidebar`, `#page-container`
        *   Pages/Content: `.pf` (page frame), `.pc` (page content), `.c` (clip), `.t` (text layer)
        *   Backgrounds: `.bf` (background frame), `.bi` (background image)
        *   Fonts: `.ff0` (sans-serif fallback), `.ff1` (primary embedded font)
    *   **Patterns:** Absolute Positioning, Inline CSS, Embedded Fonts (Base64).
    *   **Specific Fixtures:**
        *   `basic_text.html`: Basic text rendering validation.
        *   `fontfile3_opentype.html`: OpenType font rendering validation.
        *   `invalid_unicode_issue477.html`: Edge case handling for invalid Unicode.
        *   `svg_background_with_page_rotation_issue402.html`: SVG background and rotation logic.
        *   `text_visibility.html`: Text clipping and visibility logic.
        *   `with_form.html`: PDF form element rendering.
        *   `geneve_1564.html`: Complex document with custom fonts.

*   **Styling (`fancy.min.css`):**
    *   **Rule:** `#page-container { overflow: hidden; }`
    *   **Purpose:** Ensures consistent viewport dimensions during screenshot capture by preventing scrollbars.

## Architectural Patterns

Based on the provided code analysis, the following patterns are explicitly present:

1.  **Template Method:**
    *   **Evidence:** `BrowserTests` defines the general workflow (`run_test_case`), while subclasses (`test_local_browser`, `test_remote_browser_base`) implement specific steps like `generate_image` and `setUpClass`.
2.  **Golden Master:**
    *   **Evidence:** `browser_tests.py` compares rendered output against pre-existing reference images to detect regressions.
3.  **Factory:**
    *   **Evidence:** `test_remote_browser.py` uses a `generate_classes` function to dynamically instantiate test classes based on the `BROWSER_MATRIX` configuration.
4.  **Inheritance:**
    *   **Evidence:** Both `test_local_browser` and `test_remote_browser_base` inherit directly from `BrowserTests` to reuse core testing logic.

## Dependency Graph

### Internal Dependencies
*   `test_local_browser.py` imports `browser_tests` (specifically `BrowserTests`).
*   `test_remote_browser.py` imports `browser_tests` (specifically `BrowserTests`).
*   `test_output.py` imports `test` (references a `Common` class's method).

### External Dependencies
*   **PIL (Pillow):** Used for image processing and pixel difference analysis in `browser_tests.py`.
*   **Selenium:** Used for WebDriver control in `test_local_browser.py` and `test_remote_browser.py`.
*   **SauceClient:** Used for reporting results to Sauce Labs in `test_remote_browser.py`.

### Runtime Dependencies (Python Standard Library)
*   `unittest`: Test framework base.
*   `os`: File system operations (used across multiple files).
*   `subprocess`: Process execution (implied by conversion tool usage).
*   `shutil`: High-level file operations.
*   `sys`: System-level interaction (specifically for exit codes in `old/test.py`).

## Mermaid Diagram

```mermaid
classDiagram
    class BrowserTests {
        +list DEFAULT_PDF2HTMLEX_ARGS
        +run_test_case()
        +test_fail()
        +test_basic_text()
        +test_svg_background_with_page_rotation_issue402()
    }

    class test_local_browser {
        +setUpClass()
        +tearDownClass()
        +generate_image()
    }

    class test_remote_browser_base {
        +list BROWSER_MATRIX
        +map SAUCE_OPTIONS
        +generate_classes()
        +setUpClass()
        +tearDown()
        +generate_image()
    }

    class test_output {
        +run_test_case()
        +test_generate_single_html_default_name_single_page_pdf()
        +test_issue501()
    }

    class "HTML Fixtures" as HTML {
        +CSS Classes (.pf, .pc, .t, etc.)
        +Base64 Fonts
    }

    BrowserTests <|-- test_local_browser : inherits
    BrowserTests <|-- test_remote_browser_base : inherits
    test_output ..> Common : uses
    
    test_local_browser --> Selenium : uses
    test_remote_browser_base --> Selenium : uses
    test_remote_browser_base --> SauceClient : uses
    BrowserTests --> PIL : uses
    
    BrowserTests ..> HTML : validates
```