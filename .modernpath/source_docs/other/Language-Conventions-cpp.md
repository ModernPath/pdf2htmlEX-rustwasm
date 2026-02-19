# Language Conventions: cpp

## 1. Naming Conventions
*   **Classes**: Use PascalCase (e.g., `HTMLTextPage`, `StringFormatter`).
*   **Methods/Functions**: Use snake_case (e.g., `dump_text`).
*   **Source Files**: Use PascalCase for implementation files (e.g., `HTMLTextPage.cc`, `StringFormatter.cc`).
*   **Utility/Header Files**: Use lowercase or snake_case for general utility headers (e.g., `const.h`).

## 2. Code Organization
*   **Class-Per-File**: Implementation logic for specific components is encapsulated in dedicated `.cc` files matching the class name (e.g., `HTMLTextPage.cc`).
*   **Centralized Constants**: Global constants, configuration structures, and mappings are centralized in specific utility headers (e.g., `src/util/const.h`).
*   **Test Structure**: Browser-based tests are organized into a specific directory hierarchy: `test/browser_tests/[test_case_name]/[test_case_name].html`.
*   **Directory Separation**: Core logic resides in `src/`, while shared helpers and data structures are placed in `src/util/`.

## 3. Import/Module Patterns
*   No clear pattern observed (specific `#include` directives were not provided in the sample).

## 4. Error Handling
*   No clear pattern observed.

## 5. Type/Contract Patterns
*   **Configuration Objects**: Use structures within header files to define configuration and mapping contracts used across the conversion pipeline.
*   **Resource Management**: Use explicit destructors (e.g., `~HTMLTextPage`) to manage the lifecycle of conversion and optimization structures.

## 6. Idiomatic Patterns
*   **Functors for Utilities**: Use operator overloading (specifically `operator()`) to create functional utility classes, such as for string formatting (e.g., `StringFormatter`).
*   **Printf-style Formatting**: Prefer `printf`-style syntax for internal string buffer formatting and dynamic sizing.
*   **Method-Based Dumping**: Use `dump_` prefixed methods (e.g., `dump_text`) for functions responsible for outputting processed data to the final format (HTML).