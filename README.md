# WebGL2 Development Platform: VISION

A **Rust + WASM** based toolkit for debugging GLSL shaders and generating ergonomic WebGL2 bindings.

![WebGL2 Singing Dog Logo](./webgl2.png)

## 🎯 Quick Start

```bash
# Build the project
cargo build --release

# Compile a GLSL shader to WASM with debug info
cargo run --bin webgl2 -- compile tests/fixtures/simple.vert --debug

# Validate a shader
cargo run --bin webgl2 -- validate tests/fixtures/simple.frag

# Generate TypeScript harness
cargo run --bin webgl2 -- codegen tests/fixtures/simple.vert -o output.ts
```

### Build via npm

This repository is both a Rust workspace and an npm package. Run the npm build helper to build the Rust workspace for the WASM target and copy produced .wasm files into `runners/wasm/`:

```bash
# from repository root
npm run build
```

Notes:
- The script runs `cargo build --target wasm32-unknown-unknown --release` and copies any `.wasm` files from `target/wasm32-unknown-unknown/release/` into `runners/wasm/`.
- If you need `wasm-bindgen` output (JS glue), run `wasm-bindgen` manually on the produced `.wasm` files; adding automated wasm-bindgen support is a follow-up task.

## 🚀 Project Overview and Goals

The project aims to create a **Composite WebGL2 Development Platform** built with **Rust and WASM**. The primary objective is to significantly improve the developer experience by introducing standard software engineering practices—specifically, **robust debugging and streamlined resource management**—into the WebGL/GLSL workflow, which is currently hindered by platform-specific API complexity and opaque GPU execution.

| Key Goals | Target Block | Value Proposition |
| :--- | :--- | :--- |
| **GPU Debugging** | Block 1 (Emulator) | Enable **step-through debugging**, breakpoints, and variable inspection for GLSL code. |
| **Unit Testing** | Block 1 (Emulator) | Provide a stable, deterministic environment for **automated testing** of graphics logic. |
| **API Ergonomics** | Block 2 (Codegen) | **Automate boilerplate code** for resource binding, attribute setup, and uniform linking. |
| **Tech Stack** | Both | Utilize **Rust for safety and WASM for high-performance cross-platform execution** in the browser. |

-----

## 🛠️ Functional Block 1: WebGL2 Software Rendering Pipeline (The Debugger)

This block provides the **deterministic, inspectable execution environment** for GLSL logic.

### 1\. **Core Component: Rust-based WebGL2 Emulator (`wasm-gl-emu`)**

  * **State Machine Emulation:** Implement a full software model of the WebGL2 state (e.g., Framebuffer Objects, Renderbuffers, Texture Units, Vertex Array Objects, current programs, depth/stencil settings). This component will track all API calls and maintain a CPU-accessible copy of all state.
  * **Rasterization Logic:** Implement CPU-based vertex and fragment processing. This logic must precisely follow the WebGL2/OpenGL ES 3.0 specification, including clipping, primitive assembly, and the fragment pipeline (culling, depth test, blending).
  * **Input/Output:** Expose the emulator's state and rendering results via a standard Rust interface that can be compiled to WASM and wrapped for JavaScript access.

### 2\. **GLSL Translation and Debugging Integration (`glsl-to-wasm`)**

  * **GLSL Frontend:** Use a Rust-based GLSL parser (e.g., based on `naga` or a custom parser) to convert shader source code into an Intermediate Representation (IR).
  * **WASM Backend:** Compile the IR into **WASM module functions**. Each shader stage (Vertex, Fragment) will be converted into a function that executes the shader logic on a single vertex or fragment input.
  * **Source Map Generation:** The crucial step is to generate **high-quality source maps** that link the generated WASM instructions back to the original GLSL source code line and variable names. This enables DevTools/IDE to pause WASM execution and display the corresponding GLSL line and variable values.
  * **JIT Compilation:** For performance during debugging, the WASM modules can be compiled using a fast JIT compiler (potentially leveraging existing browser capabilities or a custom runtime) to execute the many vertex/fragment calls.

### 3\. **Debugging and Testing Harness**

  * **Test Runner:** Develop a testing harness in Rust/WASM that can execute a defined set of inputs (e.g., vertex attributes, uniform values) against the emulated pipeline and assert on the final pixel output or intermediate variable state.
  * **Step-Through Integration:** The WASM modules, when run by the browser's JavaScript engine, will expose the generated source maps, allowing the developer to naturally set **breakpoints** within the GLSL source file viewed in the browser's DevTools (or a connected IDE).

-----

## ⚙️ Functional Block 2: Introspection and Codegen Tool (The Ergonomics Layer)

This block automates the error-prone, repetitive JavaScript/TypeScript code required for WebGL2 resource setup.

### 1\. **GLSL Introspection Engine (`glsl-parser-rs`)**

  * **Parsing:** Use a dedicated Rust parser to analyze the GLSL source files (both Vertex and Fragment shaders).
  * **Annotation Extraction:** The parser must identify and extract **discardable annotations** embedded in the GLSL source.
      * *Example Annotation:*
        ```glsl
        //! @buffer_layout MeshData
        layout(location = 0) in vec3 a_position;
        // JSDoc-like for resource description
        /** @uniform_group Camera @semantic ProjectionMatrix */
        uniform mat4 u_projection;
        ```
  * **Resource Mapping:** Generate a structured data model (e.g., JSON or Rust structs) that lists all attributes, uniforms, uniform blocks, and their associated types, locations, and extracted metadata (like semantic hints from the annotations).

### 2\. **Code Generation Module (`js-harness-codegen`)**

  * **Harness Template:** Define a set of customizable templates (e.g., Handlebars, Tera) for generating the target **JavaScript/TypeScript harness code**.
  * **Code Generation:** Using the resource map data from the parser, the module will generate files that:
      * Define **JavaScript/TypeScript classes** for each shader program.
      * Provide methods for **binding resource objects** (e.g., `program.setUniforms(cameraData)`).
      * Implement all the necessary `gl.bindBuffer()`, `gl.getUniformLocation()`, `gl.vertexAttribPointer()`, and `gl.uniformX()` calls.
      * *Goal:* Reduce the developer's WebGL setup code to simple, type-safe resource assignments.

### 3\. **Tool Integration**

  * Package the Rust component as a **Command-Line Interface (CLI)** tool (or an accompanying library) that runs during the build step, consuming GLSL files and outputting the JavaScript/TypeScript harness code. This integrates smoothly into modern build pipelines (e.g., Webpack, Vite).

-----

## ⏳ Project Phases and Deliverables

| Phase | Duration | Focus | Key Deliverables |
| :--- | :--- | :--- | :--- |
| **Phase 1** | 3 Months | Core Compiler & Codegen | Functional GLSL parser (Block 2). CLI tool for JS harness generation (Block 2). Prototype `glsl-to-wasm` compiler (Block 1). |
| **Phase 2** | 4 Months | Core Emulator Implementation | Basic WebGL2 State Machine and Triangle Rasterizer (Block 1). Source Map integration (GLSL \<-\> WASM \<-\> DevTools) (Block 1). |
| **Phase 3** | 3 Months | Feature Completion & Polishing | Full WebGL2 feature set support in emulator (textures, complex blending, stencil). Robustness testing and documentation. |
| **Phase 4** | 2 Months | Integration & Release | Unit testing framework integration. Final documentation and developer tutorials. Full platform release. |

-----

## � Project Structure

```
webgl2/
├── crates/
│   ├── naga-wasm-backend/    # Naga IR → WASM compiler with DWARF
│   ├── wasm-gl-emu/          # Software rasterizer & WASM runtime
│   ├── glsl-introspection/   # GLSL parser + annotation extraction
│   ├── js-codegen/           # TypeScript harness generator
│   └── webgl2-cli/           # Command-line interface
├── tests/fixtures/           # Test shaders
├── docs/                     # Detailed documentation
│   ├── 1-plan.md            # Original project plan
│   └── 1.1-ir-wasm.md       # Naga IR → WASM architecture
└── external/                 # Reference implementations (naga, wgpu, servo)
```

## 🏗️ Architecture

This project uses **Naga** (from the wgpu/WebGPU ecosystem) as the shader IR, rather than building a custom IR from scratch. This significantly reduces complexity while providing a proven, well-maintained foundation.

### Key Components

1. **naga-wasm-backend**: Translates Naga IR to WebAssembly with DWARF debug information
2. **wasm-gl-emu**: Executes WASM shaders in a software rasterizer for debugging
3. **glsl-introspection**: Parses GLSL and extracts resource metadata
4. **js-codegen**: Generates ergonomic TypeScript bindings
5. **webgl2-cli**: Unified command-line tool

See [`docs/1.1-ir-wasm.md`](docs/1.1-ir-wasm.md) for detailed architecture documentation.

## 🔧 Development Status

**Current Phase: Phase 0 - Foundation Setup** ✅

- [x] Workspace structure created
- [x] Core crate skeletons implemented
- [x] Basic WASM backend (emits empty functions)
- [x] Runtime structure (wasmtime integration)
- [x] CLI tool with compile/validate/codegen/run commands
- [ ] DWARF debug information generation (in progress)
- [ ] Browser DevTools integration validation

## 📚 Documentation

- [`docs/1-plan.md`](docs/1-plan.md) - Original project proposal and plan
- [`docs/1.1-ir-wasm.md`](docs/1.1-ir-wasm.md) - Naga IR → WASM architecture (recommended reading)
- [`external/use.md`](external/use.md) - Guide to leveraging external repositories

## ⏳ Project Phases

| Phase | Duration | Focus | Status |
| :--- | :--- | :--- | :--- |
| **Phase 0** | 2 Weeks | Foundation & DWARF Validation | **In Progress** |
| **Phase 1** | 8 Weeks | Core Backend (scalars, vectors, control flow) | Not Started |
| **Phase 2** | 6 Weeks | Advanced Features (uniforms, textures, matrices) | Not Started |
| **Phase 3** | 4 Weeks | Software Rasterizer Integration | Not Started |
| **Phase 4** | 8 Weeks | Codegen Tool & Polish | Not Started |

## 🧪 Testing

```bash
# Run all tests
cargo test

# Test with a simple shader
cargo run --bin webgl2 -- compile tests/fixtures/simple.vert --debug -o output.wasm
cargo run --bin webgl2 -- run output.wasm
```

## 🤝 Contributing

This project is in early development. Contributions are welcome once Phase 0 is complete and the architecture is validated.

## 📄 License

MIT OR Apache-2.0

-----

## �💰 Resource Requirements

The project will require specialized expertise in:

  * **Rust and WASM Development**
  * **Computer Graphics / GPU Pipeline Implementation** (for the emulator)
  * **Compiler Design / Language Tooling** (for GLSL parsing and source map generation)

Would you like to discuss the **specific toolchain recommendations** for the GLSL-to-WASM compilation process?