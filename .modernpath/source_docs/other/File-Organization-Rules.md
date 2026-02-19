Based on the provided directory structure for **pdf2htmlEX**, the file organization rules are as follows:

### 1. Directory Structure
*   **Three-Tier Top-Level:** The project is divided into three primary areas: external dependencies (`3rdparty`), core logic (`src`), and validation (`test`).
*   **Functional Modularization:** The `src` directory is organized by functional responsibility (e.g., different types of renderers and utilities).
*   **Scenario-Based Testing:** The `test` directory uses a nested structure to isolate specific test cases or bug regressions.

### 2. File Naming
*   **No clear pattern observed.** (Specific file names were not provided in the data).

### 3. File Placement
*   **External Dependencies:** Third-party libraries and external codebases are placed in the `3rdparty/` directory (e.g., `PDF.js`).
*   **Core Logic:** Implementation code is located within `src/`.
*   **Renderer Logic:** Code responsible for specific output formats is isolated into dedicated subdirectories: `src/BackgroundRenderer` and `src/HTMLRenderer`.
*   **Shared Utilities:** Common helper functions or shared logic are located in `src/util`.
*   **Testing:** All test-related materials are kept in the `test/` directory, separate from the source code.

### 4. Related Files
*   **Test Case Grouping:** Browser-based tests are grouped into individual subdirectories under `test/browser_tests/`. These groups are organized by:
    *   **Feature:** (e.g., `basic_text`, `text_visibility`)
    *   **Issue Number:** (e.g., `invalid_unicode_issue477`, `svg_background_with_page_rotation_issue402`)
    *   **Specific File Types/Scenarios:** (e.g., `fontfile3_opentype`, `with_form`)

### 5. Special Directories
*   **`3rdparty/`**: Reserved for vendor code and external projects integrated into the build.
*   **`src/util/`**: Dedicated to reusable utility modules used across the different renderers.
*   **`test/browser_tests/`**: Contains specific integration or regression tests designed to be run in a browser environment.
*   **`test/old/`**: Contains legacy or deprecated test suites.