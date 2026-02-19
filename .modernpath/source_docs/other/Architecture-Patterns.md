Based on the provided subsystem information for **pdf2htmlEX**, the following architectural patterns are evidenced:

### 1. Overall Architecture
*   **Monolithic Architecture**: The system is structured as a single integrated application consisting of specialized engines (Background and HTML) and a central orchestration logic. There is no evidence of distributed services or microservices.

### 2. Layer Structure
The system follows a **Functional Layering** approach:
*   **Orchestration Layer**: Represented by the "Main Application Logic," which serves as the entry point and coordinates the execution flow.
*   **Processing/Domain Layer**: Comprised of the "Background Rendering Engine" and "HTML Rendering Engine," which contain the core logic for transformation.
*   **Abstraction Layer**: The "External Dependencies" subsystem (e.g., PDF.js) acts as a boundary layer for underlying PDF processing.
*   **Support Layer**: "Core Utilities" provides shared logic across all other layers.

### 3. Component Communication
*   **Centralized Orchestration (Controller)**: The "Main Application Logic" acts as a controller that coordinates the "rendering pipeline." Communication is likely synchronous and directed from the Main Logic to the specialized Rendering Engines.
*   **Shared Library Access**: The "Core Utilities" are accessed by multiple subsystems, indicating a many-to-one communication pattern for helper functions and data structures.

### 4. Data Flow Patterns
*   **Transformation Pipeline**: The system follows a linear pipeline pattern. Data flows from the **External Dependencies** (raw PDF processing) → **Main Application Logic** (orchestration) → **Rendering Engines** (transformation to canvas/images and semantic HTML) → **Final Output**.

### 5. Extension Points
*   **Engine Modularity**: The separation of the "Background Rendering Engine" and "HTML Rendering Engine" indicates that the system is designed to allow independent updates or extensions to how visual backgrounds versus semantic structures are handled.
*   **External Integration**: The "External Dependencies" subsystem suggests a specific point for swapping or updating underlying PDF parsing libraries (like PDF.js).

### 6. Critical Paths
*   **The Rendering Pipeline**: The coordination logic within the "Main Application Logic" that sequences the Background and HTML rendering engines is the most critical path, as it manages the lifecycle of the conversion process.
*   **External Library Integration**: The dependency on external PDF processing (PDF.js) is a critical integration point that the rest of the system relies upon for data input.

***

**Note on Missing Information:**
*   **Data Models**: Because data model information was not available, specific patterns regarding state management (e.g., Active Record, Data Mapper) or internal data representation cannot be determined.
*   **Dependencies**: Detailed dependency mapping between specific files is unavailable, so the exact coupling level between engines cannot be confirmed.