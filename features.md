# ✅ Implemented Features
These features have implementations in both Rust and JavaScript, and appear to be functional.

*   **Context & State**
    *   `getError`, `enable`, `disable`
    *   `viewport`, `scissor`
    *   `clear`, `clearColor`, `depthFunc`
    *   `activeTexture`
*   **Textures**
    *   `createTexture`, `deleteTexture`, `bindTexture`
    *   `texImage2D` (Basic 2D texture upload)
    *   `texParameteri` (Texture parameters)
*   **Framebuffers**
    *   `createFramebuffer`, `deleteFramebuffer`, `bindFramebuffer`
    *   `framebufferTexture2D` (Attaching textures to framebuffers)
    *   `readPixels` (Reading back pixel data)
*   **Shaders & Programs**
    *   `createShader`, `shaderSource`, `compileShader`, `deleteShader`
    *   `getShaderParameter`, `getShaderInfoLog`
    *   `createProgram`, `attachShader`, `linkProgram`, `deleteProgram`, `useProgram`
    *   `getProgramParameter`, `getProgramInfoLog`
*   **Buffers & Attributes**
    *   `createBuffer`, `bindBuffer`, `deleteBuffer`
    *   `bufferData` (Uploading data to buffers)
    *   `bufferSubData` (Updating buffer data)
    *   `getBufferParameter`
    *   `getAttribLocation`, `bindAttribLocation`
    *   `enableVertexAttribArray`, `disableVertexAttribArray`
    *   `vertexAttribPointer`
    *   `vertexAttrib1f`, `vertexAttrib2f`, `vertexAttrib3f`, `vertexAttrib4f`
*   **Uniforms**
    *   `getUniformLocation`
    *   `uniform1f`, `uniform2f`, `uniform3f`, `uniform4f`
    *   `uniform1i`
    *   `uniformMatrix4fv`
*   **Drawing**
    *   `drawArrays`
    *   `drawElements`

# ⚠️ Partially Implemented
*   **`getParameter`**: Only supports `VIEWPORT` and `COLOR_CLEAR_VALUE`. All other parameters throw an error.

# ❌ Not Implemented (Stubs)
These functions exist in the API surface but explicitly throw a "not implemented" error when called.

*   **Vertex Array Objects (VAOs)**: `createVertexArray`, `bindVertexArray`, `deleteVertexArray`, `isVertexArray`.
*   **Instanced Drawing**: `drawArraysInstanced`, `drawElementsInstanced`.
*   **Advanced Texture Operations**: `generateMipmap`, `copyTexImage2D`, `copyTexSubImage2D`, `texImage3D`.
*   **Advanced Buffer Operations**: `copyBufferSubData`, `isBuffer`.
*   **Renderbuffers**: All functions (`createRenderbuffer`, `bindRenderbuffer`, etc.).
*   **Transform Feedback**: All functions.
*   **Queries & Sync**: All functions (`createQuery`, `fenceSync`, etc.).
*   **Samplers**: All functions (`createSampler`, etc.).
*   **Blending & Stencil**: `blendFunc`, `blendEquation`, `stencilFunc`, `stencilOp`, etc.
*   **Depth & Masking**: `clearDepth`, `depthMask`, `colorMask`.
*   **Introspection**: `getActiveUniform`, `getActiveAttrib`, `getExtension`, `getSupportedExtensions`.
*   **Misc**: `pixelStorei`, `finish`, `flush`, `polygonOffset`, `sampleCoverage`.
*   **Object Queries**: `isTexture`, `isFramebuffer`, `isProgram`, `isShader`, `isEnabled`.

# 🧪 Test Coverage
The test directory contains test files for almost every WebGL2 function, including the unimplemented ones. This suggests the test suite is set up to verify both working features and correct error handling (or placeholders) for missing features.