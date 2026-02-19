# Interfaces Documentation: test

## Overview
The `test` module serves as a comprehensive integration and validation suite for the `pdf2htmlEX` conversion tool. Rather than exposing a REST/GraphQL API, this module acts as a client that integrates with external command-line tools, browser automation APIs, and cloud-based testing services.

The primary integration points include:
*   **Command-Line Interface (CLI):** Execution of the `pdf2htmlEX` binary.
*   **Browser Automation:** Interaction with local and remote browsers via the Selenium WebDriver API.
*   **Cloud Services:** Reporting and session management with the Sauce Labs API.

## External Service Integrations

### Sauce Labs Remote Browser Service
The module integrates with Sauce Labs to execute visual regression tests across multiple browser environments.

*   **Module:** `test/test_remote_browser.py`
*   **Library:** `sauceclient`
*   **Authentication Mechanism:** Environment variables.
    *   `USERNAME`: Sauce Labs username.
    *   `ACCESS_KEY`: Sauce Labs access key.
*   **Integration Workflow:**
    1.  **Session Initialization:** The `test_remote_browser_base.setUpClass` method initializes a remote WebDriver instance with specific capabilities (OS, browser name, version) defined in `BROWSER_MATRIX`.
    2.  **Result Reporting:** The `test_remote_browser_base.tearDown` method updates the Sauce Labs job status. It reports pass/fail results, build numbers, and tags.
    3.  **Metadata Tagging:** Tags are derived from Travis CI environment variables (e.g., `TRAVIS_PULL_REQUEST` or `TRAVIS_BRANCH`).
*   **Configuration:**
    *   `BROWSER_MATRIX`: A list of tuples defining target operating systems, browser names, and versions.
    *   `SAUCE_OPTIONS`: Configuration dictionary for session options (e.g., video recording).

## Browser Automation Protocols

The module utilizes the Selenium WebDriver API to render HTML content and capture visual data.

### Local Browser Integration
*   **Module:** `test/test_local_browser.py`
*   **Protocol:** `file://` protocol.
*   **Implementation Details:**
    *   **WebDriver:** Initializes a local Firefox instance.
    *   **Window Management:** Explicitly resizes the browser window to `BROWSER_WIDTH` and `BROWSER_HEIGHT` to ensure consistent rendering dimensions.
    *   **Page Loading:** The `generate_image` method loads HTML files via the `file://` protocol.
    *   **Error Handling:** Catches `WebDriverException` during page loads; behavior depends on the `page_must_load` flag.

### Remote Browser Integration
*   **Module:** `test/test_remote_browser.py`
*   **Implementation Details:**
    *   **WebDriver:** Configures remote WebDriver instances via Sauce Labs.
    *   **Page Loading:** The `generate_image` method navigates to a specific URL and waits for the `#page-container` element to load before capturing a screenshot.

## Command-Line Interface Integration

The module functions as a client for the `pdf2htmlEX` command-line tool, invoking it to generate test artifacts.

### pdf2htmlEX Execution
*   **Modules:** `test/browser_tests.py`, `test/test_output.py`, `test/old/test.py`
*   **Invocation Methods:**
    *   **Subprocess/Wrapper:** `test/browser_tests.py` and `test/test_output.py` execute the tool (the latter via a `Common` class's `run_pdf2htmlEX` method) to perform conversions with specific arguments (e.g., `fit-width`, `last-page`).
    *   **System Shell:** `test/old/test.py` uses `os.system` to execute the command directly.
*   **Input/Output Contract:**
    *   **Input:** PDF files located in specific test directories.
    *   **Output:** HTML files (single or split pages) and associated assets.
    *   **Error Handling:** `test/old/test.py` checks the return code of `os.system` and halts execution (`sys.exit(-1)`) if a conversion fails.

## Internal Component Interfaces

### BrowserTests Abstract Class
*   **Module:** `test/browser_tests.py`
*   **Pattern:** Template Method.
*   **Interface Contract:** Defines a base class for visual regression testing. It requires subclasses to implement specific rendering logic while providing the core workflow (conversion, comparison, reporting).
*   **Abstracted Method:**
    *   `generate_image`: Must be implemented by subclasses (e.g., `test_local_browser`, `test_remote_browser`) to handle the specific mechanism of loading a page and capturing a screenshot (file vs. remote URL).

### File System Interface
*   **Modules:** `test/browser_tests.py`, `test/test_output.py`
*   **Operations:**
    *   **Read/Write:** Heavy reliance on file system operations to read pre-compiled HTML, write generated HTML, and save reference/comparison images (PNG).
    *   **Fallback Logic:** `browser_tests.py` implements logic to check for pre-compiled files in `PREDIR` and falls back to generating them if missing.