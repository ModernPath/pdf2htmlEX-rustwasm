# Use Cases Documentation: Core Utilities

## Subsystem Overview
The Core Utilities subsystem provides shared helper functions, data structures, and common logic used across the application. It acts as a foundational layer supporting other application components by handling low-level operations and cross-cutting concerns.

## User Workflows & Business Processes
**Status:** Not determined from available documentation.

The provided module documentation for `src/util` and `src` contains only section titles and does not provide specific details regarding user-facing workflows, UI routes, controller actions, or business processes. Therefore, specific user journeys, roles, or business process flows cannot be documented for this subsystem based on the provided data.

## Subsystem Capabilities
While direct user workflows are not defined in the provided documentation, the subsystem explicitly supports the following internal capabilities and responsibilities:

*   **Filesystem and Path Manipulation**
    *   Supports the creation, sanitization, and parsing of filesystem paths.
*   **Unicode and Encoding Handling**
    *   Supports Unicode mapping and validation.
    *   Supports encoding for HTML, JSON, and UTF-8.
*   **Geometric Calculations**
    *   Supports matrix transformations.
    *   Supports bounding box intersections.
*   **Font Processing**
    *   Supports FontForge library lifecycle management.
    *   Supports font processing via a C-wrapper.
*   **Platform Abstraction**
    *   Supports platform-specific abstraction, specifically for MinGW/Windows compatibility.
    *   Supports signal handling.

## Architectural Decisions
*   **Abstraction Strategy:** The subsystem utilizes platform-specific abstraction to handle compatibility issues between environments such as MinGW and Windows.
*   **Integration Strategy:** Font processing is handled via a C-wrapper, indicating a decision to interface with native C libraries (FontForge) rather than a pure implementation.

## External Dependencies and Integrations
*   **FontForge:** The subsystem integrates with the FontForge library for font processing, managed via a C-wrapper.