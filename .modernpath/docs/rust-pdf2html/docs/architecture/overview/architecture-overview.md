# Architecture Overview

**Tier:** architecture
**Angle:** overview


---

# Architecture Overview: rust-pdf2html

## 1. System Overview

### Purpose and Business Context
**rust-pdf2html** is a modernized, high-performance document conversion engine designed to transform PDF files into semantic HTML5. It serves as the next-generation evolution of legacy conversion tools, specifically engineered for the modern web ecosystem.

The system operates within the **Document Processing and Digital Archiving** domain. Its primary business purpose is to enable organizations to publish static PDF documents as interactive, searchable, and responsive web content without relying on heavy browser plugins or sacrificing visual fidelity. By bridging the gap between fixed-layout archival formats (PDF) and dynamic web standards, the system facilitates broader document accessibility and distribution.

### Problem Statement
Legacy document conversion systems often suffer from several critical limitations:
*   **Security Vulnerabilities:** Reliance on memory-unsafe languages (C/C++) for parsing complex file formats leads to risks of buffer overflows and exploitation.
*   **Performance Bottlenecks:** Synchronous processing models limit throughput for high-volume batch operations.
*   **Poor Web Integration:** Generated HTML often lacks semantic structure or requires heavy, non-standard polyfills to render correctly across browsers.
*   **Rigid Deployment:** Monolithic architectures make it difficult to scale processing independently or deploy rendering capabilities to the client-side.

**rust-pdf2html** addresses these issues by leveraging a memory-safe core, asynchronous processing, and a hybrid execution model supporting both server-side and client-side (WebAssembly) rendering.

---

## 2. High-Level Architecture

The system follows a **Hybrid Layered Architecture** with a focus on separation of concerns. It is designed to support two distinct execution paths:
1.  **Server-Side Batch Processing:** High-throughput conversion via a REST API.
2.  **Client-Side Rendering:** In-browser conversion using WebAssembly (Wasm) for low-latency viewing.

```mermaid
graph TD
    subgraph "Client Layer"
        ReactUI[React UI / Viewer]
        WasmModule[Rust Core (Wasm)]
    end

    subgraph "API Layer"
        AxumServer[Axum Web Server]
        TaskQueue[Redis Task Queue]
    end

    subgraph "Processing Layer"
        Worker[Async Worker (Tokio)]
        PDFEngine[PDF Conversion Engine]
    end

    subgraph "Data Layer"
        Postgres[(PostgreSQL - Metadata)]
        Storage[(Object Storage - Artifacts)]
    end

    %% Interactions
    ReactUI <-->|Local Conversion| WasmModule
    ReactUI -->|REST API / Upload| AxumServer
    AxamServer -->|Queue Job| TaskQueue
    Worker -->|Dequeue Job| TaskQueue
    Worker -->|Read/Write| Postgres
    Worker -->|Store HTML/Assets| Storage
    Worker -->|Invoke| PDFEngine
    AxumServer -->|Query Status| Postgres
```

### Component Descriptions

*   **PDF Conversion Engine (Rust Core):**
    *   The heart of the system, written in Rust. It handles low-level PDF parsing, font extraction, graphics state management, and DOM reconstruction.
    *   Designed as a library crate that can be compiled to native binaries or WebAssembly.
    *   **Responsibilities:** Vector path rasterization, font subsetting/embedding, and mapping PDF coordinate systems to CSS layouts.

*   **Client Layer (React & Wasm):**
    *   **React UI:** A component library providing a document viewer interface. It handles user interactions, zooming, and page navigation.
    *   **Wasm Module:** The Rust core compiled to WebAssembly. This allows the browser to perform PDF-to-HTML conversion locally, offloading the server and ensuring privacy (files do not leave the client device).

*   **API Layer (Axum & Redis):**
    *   **Axum Server:** A high-performance HTTP server providing REST endpoints for uploading PDFs, checking job status, and retrieving converted HTML.
    *   **Redis:** Acts as a message broker and task queue. It decouples the HTTP API from the heavy processing workers, allowing the system to absorb traffic spikes.

*   **Processing Layer (Tokio Workers):**
    *   Background workers built on **Tokio**, Rust's asynchronous runtime.
    *   These workers poll Redis for jobs, execute the conversion engine, and handle the storage of results. They are stateless and can be scaled horizontally.

*   **Data Layer:**
    *   **PostgreSQL:** Stores relational metadata such as job status, conversion timestamps, user ownership, and processing logs.
    *   **Object Storage:** Stores the original PDF files and the generated HTML assets (images, fonts, CSS).

---

## 3. Key Design Decisions

### Technology Stack Rationale

| Technology | Justification |
| :--- | :--- |
| **Rust** | **Memory Safety & Performance:** PDF parsing is prone to security exploits. Rust’s ownership model ensures memory safety without garbage collection, providing C++-like performance with modern safety guarantees. |
| **WebAssembly (Wasm)** | **Client-Side Execution:** Enables the core engine to run natively in the browser at near-native speed. This reduces server costs and allows for offline document viewing. |
| **Axum & Tokio** | **Async Concurrency:** Handling thousands of concurrent file uploads and conversions requires efficient I/O. Tokio's non-blocking runtime allows the system to scale efficiently on limited hardware. |
| **React & Tailwind** | **Ecosystem & UX:** React provides a robust component model for the viewer, while Tailwind allows for rapid, consistent UI development. |
| **PostgreSQL & Redis** | **Reliability & Speed:** Postgres offers ACID compliance for metadata, while Redis provides the low-latency queuing mechanism required for high-throughput job processing. |

### Architectural Patterns

*   **Strategy Pattern:** The rendering engine utilizes interchangeable strategies for background rendering (e.g., pure SVG vs. Hybrid Bitmap fallback) depending on the complexity of the PDF page.
*   **Repository Pattern:** Abstractions over the Data Layer allow switching between local filesystem storage and cloud object storage (S3) without impacting business logic.
*   **Actor/Model Concurrency:** The Tokio workers act as independent actors processing messages from the Redis queue, ensuring isolation and fault tolerance.

### Trade-offs

*   **Wasm Binary Size vs. Functionality:** Compiling the full Rust standard library and rendering engine to Wasm results in a larger initial payload. We mitigate this by aggressive code pruning and feature flagging (e.g., enabling only necessary font formats).
*   **Client-Side vs. Server-Side Rendering:** While client-side rendering saves server resources, it relies on the client's CPU. For very large documents (e.g., 1000+ pages), the system defaults to server-side processing to prevent browser crashes.

---

## 4. Quality Attributes

### Scalability
The system is designed for horizontal scalability.
*   **Stateless Workers:** Conversion workers do not maintain local state. New worker instances can be added to the cluster simply by pointing them at the Redis and Postgres instances.
*   **Asynchronous Processing:** The API offloads work immediately to Redis, preventing HTTP threads from blocking during long conversion processes.

### Performance
*   **Zero-Copy Abstractions:** Where possible, the Rust codebase uses zero-copy parsing to minimize memory allocation during PDF stream processing.
*   **Parallelism:** Multi-page PDFs are processed in parallel chunks where the document structure allows, maximizing CPU utilization.

### Security Model
*   **Sandboxing (Wasm):** Client-side execution runs in the browser's sandbox, preventing any access to the user's filesystem or network.
*   **Input Validation:** All uploaded files are strictly validated (magic bytes, header checks) before being passed to the parsing engine.
*   **Memory Safety:** The use of Rust eliminates entire classes of vulnerabilities related to buffer overflows and dangling pointers common in previous C++ implementations.

---

## 5. Getting Started

### Prerequisites
To build and run `rust-pdf2html`, ensure you have the following installed:

*   **Rust Toolchain:** Stable Rust 1.70 or later (`rustup`, `cargo`).
*   **Node.js & npm:** Version 18+ (for the UI component).
*   **Docker & Docker Compose:** For running PostgreSQL and Redis locally.
*   **Wasm-Pack:** For building the WebAssembly module (`cargo install wasm-pack`).

### Quick Start Guide

1.  **Clone the Repository:**
    ```bash
    git clone https://github.com/your-org/rust-pdf2html.git
    cd rust-pdf2html
    ```

2.  **Start Infrastructure:**
    Start the database and cache services using Docker Compose:
    ```bash
    docker-compose up -d
    ```

3.  **Build the Rust Core (Server & Wasm):**
    ```bash
    # Build the native server binary
    cargo build --release

    # Build the Wasm module for the browser
    cd wasm-lib
    wasm-pack build --target web
    cd ..
    ```

4.  **Build the UI:**
    ```bash
    cd ui
    npm install
    npm run build
    cd ..
    ```

5.  **Run the Server:**
    ```bash
    # Run the API and Worker
    cargo run --bin server
    ```

6.  **Access the Application:**
    *   **API Documentation:** Open `http://localhost:3000/swagger-ui` for interactive API docs.
    *   **React Viewer:** Open `http://localhost:3000` to upload and view documents.

### Running Tests
Execute the full test suite to verify functionality:
```bash
# Rust unit and integration tests
cargo test

# UI tests
cd ui && npm test
```


---

## Summary

High-level overview of the rust-pdf2html architecture
