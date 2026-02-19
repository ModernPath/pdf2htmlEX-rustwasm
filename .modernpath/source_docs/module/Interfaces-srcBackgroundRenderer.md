# Interfaces Documentation: `src/BackgroundRenderer`

## Overview
The `src/BackgroundRenderer` module provides a C++ API for rendering PDF page backgrounds into various image formats (SVG, PNG, JPEG). It acts as a subsystem within a larger document processing pipeline, abstracting the complexity of vector and raster graphics libraries (Cairo and Poppler Splash) behind a unified interface.

The module does not expose network endpoints (REST/GraphQL) or WebSocket protocols. Instead, its integration points are defined by C++ class interfaces, factory methods for instantiation, and file system interactions for output generation.

## Core API: Factory Methods

The primary entry point for the module is the `BackgroundRenderer` abstract base class, which provides static factory methods to instantiate specific rendering implementations based on configuration.

### `BackgroundRenderer::getBackgroundRenderer`
*   **Type:** Static Factory Method
*   **Purpose:** Instantiates a concrete renderer based on the requested format string and compile-time feature flags.
*   **Input Parameters:**
    *   `format`: A string specifying the desired output format (e.g., `'png'`, `'jpg'`, `'svg'`).
    *   `param`: Configuration parameters (referenced as `Param` struct/class).
    *   `html_renderer`: A pointer to an `HTMLRenderer` instance for coordination.
*   **Return Type:** `std::unique_ptr<BackgroundRenderer>`
*   **Error Handling:** Returns `nullptr` if no format matches or if the requested feature is disabled at compile time.
*   **Logic:**
    *   Returns `SplashBackgroundRenderer` for 'png' or 'jpg' formats.
    *   Returns `CairoBackgroundRenderer` for 'svg' format.

### `BackgroundRenderer::getFallbackBackgroundRenderer`
*   **Type:** Static Factory Method
*   **Purpose:** Provides a fallback rendering strategy if the primary format fails or is unsuitable (e.g., if SVG complexity is too high).
*   **Input Parameters:** (Same as `getBackgroundRenderer`)
*   **Return Type:** `std::unique_ptr<BackgroundRenderer>`
*   **Logic:**
    *   If the background format is 'svg' and the `svg_node_count_limit` configuration is greater than or equal to 0, this factory returns a `SplashBackgroundRenderer` (bitmap) as a fallback.

## Core Renderer Interface

All renderers inherit from the `BackgroundRenderer` abstract base class, enforcing a specific contract for rendering operations.

### `init`
*   **Visibility:** Public
*   **Purpose:** Initializes the renderer with the source PDF document.
*   **Input:** PDF document object.
*   **Return Type:** `void`
*   **Side Effects:** Updates internal state to prepare for rendering.

### `render_page`
*   **Visibility:** Public
*   **Purpose:** Renders a specific page from the PDF document.
*   **Input:** Page index/identifier.
*   **Return Type:** `bool`
*   **Side Effects:** File I/O (writes background files), State changes.
*   **Return Semantics:**
    *   **CairoBackgroundRenderer:** Returns `true` if SVG was used successfully; returns `false` if a fallback to a bitmap renderer is required (e.g., if SVG node count exceeds limits).
    *   **SplashBackgroundRenderer:** Returns `true` (always).

### `embed_image`
*   **Visibility:** Public
*   **Purpose:** Embeds the rendered background image into the final output.
*   **Return Type:** `void`
*   **Side Effects:** File I/O, State changes.
*   **Behavior:**
    *   **CairoBackgroundRenderer:** Generates an HTML tag (`<img>` or `<embed>`) for the SVG background. Handles base64 embedding or external file references.
    *   **SplashBackgroundRenderer:** Generates an HTML `<img>` tag for the raster background. Embeds as Base64 data URI if configured, or saves as an external file.

## Implementation-Specific Interfaces

### CairoBackgroundRenderer (Vector/SVG)
Extends `CairoOutputDev` to provide vector-based rendering.

*   **`build_bitmap_path`**
    *   **Visibility:** Public
    *   **Purpose:** Constructs the file path for an extracted bitmap based on its stream ID.
    *   **Return Type:** `std::string`
*   **`interpretType3Chars`**
    *   **Visibility:** Public
    *   **Purpose:** Determines if Type 3 fonts should be interpreted or drawn as characters.
    *   **Return Type:** `bool`
*   **`setMimeData`**
    *   **Visibility:** Public
    *   **Purpose:** Intercepts image data during rendering to extract specific JPEGs to external files rather than embedding them in the SVG.
    *   **Integration Point:** Hooks into the Cairo library's data stream.

### SplashBackgroundRenderer (Raster/PNG/JPG)
Extends `SplashOutputDev` to provide bitmap-based rendering.

*   **`startPage`**
    *   **Visibility:** Public
    *   **Purpose:** Overrides the base method to start a new page, specifically preventing the default full-page background paint.
    *   **Return Type:** `void`
*   **`interpretType3Chars`**
    *   **Visibility:** Public
    *   **Purpose:** Determines if Type 3 fonts should be interpreted.
    *   **Return Type:** `bool`

## Integration Points

### Internal Component Integration
*   **HTMLRenderer:** Both `CairoBackgroundRenderer` and `SplashBackgroundRenderer` take a pointer to an `HTMLRenderer` in their constructors. This is used to coordinate rendering state and check text coverage/visibility.
*   **Param:** Both renderers accept a `Param` object (struct/class) which dictates runtime behavior (e.g., format selection, embedding options, node limits).

### External Library Protocols
*   **Poppler (CairoOutputDev / SplashOutputDev):** The module integrates deeply with the Poppler library by inheriting from its output device classes. It overrides virtual methods such as `drawChar`, `beginTextObject`, `endTextObject`, and `updateRender` to inject custom logic (e.g., proof visualization, text visibility checks).
*   **Cairo Library:** The Cairo renderer uses `cairo_svg_surface` and `cairo_t` contexts to generate vector graphics.

### Callback Protocols
*   **`annot_cb` (Static Callback):**
    *   **Purpose:** A static callback function used by the rendering engines to determine whether to process a specific PDF annotation.
    *   **Signature:** Returns `bool`.
    *   **Visibility:** Private (implementation detail).

### File System Interface
*   **Output:** The module writes rendered content to the file system (SVG, PNG, JPG files).
*   **Temporary Resources:** `CairoBackgroundRenderer` manages temporary bitmap files, tracking reference counts to clean up files with zero references in the destructor.

## Error Handling
The module utilizes exception handling and return codes to communicate errors:
*   **Exceptions:** Throws `std::string` exceptions for critical failures, such as Cairo errors, file I/O failures, or unsupported formats.
*   **Null Returns:** The factory method `getBackgroundRenderer` returns `nullptr` if it cannot instantiate a renderer for the requested format.
*   **Status Codes:** `CairoBackgroundRenderer::render_page` returns `false` to signal that the rendering failed or exceeded constraints (node count limit), triggering a fallback workflow.