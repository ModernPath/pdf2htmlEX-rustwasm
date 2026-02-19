# Data Models Documentation

## Module Overview
The `src/BackgroundRenderer` module is responsible for the data transformation and persistence of PDF page content into various image and vector formats. It functions as a rendering engine that converts raw PDF document data into persistent file-based outputs (SVG, PNG, JPEG) or in-memory encoded data streams (Base64). The module manages the extraction and lifecycle of embedded bitmap resources found within source PDFs.

## Data Structures

### Configuration Data
*   **`Param`**
    *   **Type:** `struct` / `class`
    *   **Description:** Configuration parameters passed to renderers to control behavior.
    *   **Status:** Referenced but not defined within this module. Specific fields are not visible in the provided code.

### External Library Structures
*   **`SplashColor`**
    *   **Type:** `struct`
    *   **Description:** A color structure utilized by the Splash graphics library for raster rendering operations.
    *   **Status:** Defined by the external Splash library; used within `SplashBackgroundRenderer`.

### Internal State Management
*   **Bitmap Reference Tracking**
    *   **Implementation:** `std::unordered_map` (referenced in dependencies)
    *   **Context:** Used within `CairoBackgroundRenderer` to track reference counts for extracted bitmap files to manage cleanup and lifecycle.
*   **Vector Storage**
    *   **Implementation:** `std::vector` (referenced in dependencies)
    *   **Context:** Used internally within `CairoBackgroundRenderer` for managing collections of rendering data or nodes.

## Persistence & File Schemas

The module does not utilize a database but relies heavily on file system persistence and data stream encoding.

### Input Data Sources
*   **PDF Documents:** The primary data source, processed via the Poppler library.

### Output Data Schemas (Formats)

#### 1. Vector Graphics (SVG)
*   **Produced by:** `CairoBackgroundRenderer`
*   **Method:** `render_page`
*   **Description:** Generates vector-based SVG files representing the PDF page background.
*   **Constraints:**
    *   If the SVG node count exceeds a configured limit (`svg_node_count_limit`), the generation is aborted, and the system signals a fallback to a bitmap renderer.

#### 2. Raster Images (PNG/JPEG)
*   **Produced by:** `SplashBackgroundRenderer`
*   **Method:** `render_page`
*   **Description:** Generates raster bitmap files.
*   **Supported Formats:** PNG and JPEG (support depends on compilation flags).
*   **Persistence Strategy:**
    *   **External File:** Saved to disk if the `embed_image` parameter is false.
    *   **Base64 Data URI:** Embedded directly into the output stream if the `embed_image` parameter is true.

#### 3. Extracted Embedded Bitmaps (JPEG)
*   **Produced by:** `CairoBackgroundRenderer`
*   **Method:** `setMimeData` (extraction logic), `build_bitmap_path` (path construction)
*   **Description:** Extracts embedded images from the PDF source to external files to avoid bloating the SVG output.
*   **File Naming:** Constructed dynamically based on the stream ID of the bitmap.
*   **Lifecycle:** Temporary files are tracked via reference counts and cleaned up in the destructor when counts reach zero.

## Data Constraints & Validation

The module applies specific logic to determine *how* data is extracted and persisted:

### Bitmap Extraction Constraints
Within `CairoBackgroundRenderer::setMimeData`, embedded bitmaps are only extracted to external files if **all** the following conditions are met:
1.  The stream is a DCT (JPEG) stream.
2.  The color space is DeviceRGB or DeviceGray.
3.  There is no Decode array present.

### Rendering Fallback Logic
*   **Node Count Limit:** In `CairoBackgroundRenderer::render_page`, the process checks if the SVG node count exceeds `svg_node_count_limit`. If exceeded, it returns `false`, triggering a fallback to the bitmap renderer.

### Text Visibility Constraints
*   **Splash Renderer:** In `SplashBackgroundRenderer::drawChar`, character data is only rendered into the bitmap if in "proof mode" or if the character is covered by the HTML renderer's text layer.

## Data Relationships

### Dependencies
*   **`BackgroundRenderer` (Abstract Base)** relies on `Param` for configuration.
*   **`CairoBackgroundRenderer`** depends on `HTMLRenderer` to coordinate the overall rendering pipeline.
*   **`SplashBackgroundRenderer`** depends on `HTMLRenderer` for text layer coverage checks.

### Factory Instantiation
The `BackgroundRenderer` class acts as a factory for data persistence strategies:
*   **`getBackgroundRenderer`**: Instantiates `SplashBackgroundRenderer` for 'png'/'jpg' formats or `CairoBackgroundRenderer` for 'svg' formats.
*   **`getFallbackBackgroundRenderer`**: Instantiates `SplashBackgroundRenderer` if the primary format is 'svg' and a node limit is configured.