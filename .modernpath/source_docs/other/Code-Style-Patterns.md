Based on the provided file descriptions and metadata, the following coding style patterns are observable:

### 1. Formatting
No clear pattern observed (source code content not provided).

### 2. Documentation Style
No clear pattern observed (comment blocks or docstrings not provided).

### 3. Function Design
*   **Naming Convention**: Member functions use `snake_case` (e.g., `dump_text`).
*   **Functional Objects**: Use operator overloading (specifically `operator()`) for classes dedicated to single-purpose utility tasks like string formatting (e.g., `StringFormatter::operator()`).
*   **Lifecycle Management**: Explicitly define constructors and destructors for primary processing classes (e.g., `HTMLTextPage`).

### 4. Module/Class Design
*   **Class Naming**: Use `PascalCase` for class names (e.g., `HTMLTextPage`, `StringFormatter`).
*   **File Extensions**: Use `.cc` for C++ implementation files and `.h` for header files.
*   **Directory Structure**: 
    *   Place core logic in a `src/` directory.
    *   Isolate utility-wide constants, mappings, and configuration structures into a specific header file located at `src/util/const.h`.
*   **Test Organization**: 
    *   Group browser-based test fixtures in `test/browser_tests/`.
    *   Use descriptive, issue-specific subdirectories for test cases (e.g., `geneve_1564`, `invalid_unicode_issue477`).
    *   Include a `README.md` in the `test/` directory to document dependencies and procedures.

### 5. API Design
*   **Public Interface**: Use classes to encapsulate specific stages of the conversion process (e.g., `HTMLTextPage` for text structure optimization).
*   **Centralized Configuration**: Use a centralized header (`const.h`) to manage global constants rather than hardcoding values within individual modules.