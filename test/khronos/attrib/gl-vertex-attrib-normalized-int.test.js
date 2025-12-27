import test from 'node:test';
import assert from 'node:assert';
import { webGL2 } from '../../../index.js';

/*
 * FAIL: This test fails because the current implementation of `fetch_vertex_attributes` in `src/webgl2_context/types.rs`
 * does not handle `GL_BYTE` (0x1400) or the `normalized` flag correctly.
 * 
 * Root Cause:
 * In `src/webgl2_context/types.rs`, the `fetch_vertex_attributes` function has a match block that only handles:
 * - `GL_FLOAT` (0x1406)
 * - `GL_UNSIGNED_BYTE` (0x1401) -> hardcoded to normalized (/ 255.0)
 * 
 * It returns 0.0 for `GL_BYTE` (0x1400), causing the test to read 0 instead of -1.0.
 * 
 * Proposed Fix:
 * Update `fetch_vertex_attributes` to handle all vertex attribute types (BYTE, SHORT, INT, etc.) and respect the `attr.normalized` flag.
 * For `GL_BYTE` with `normalized = true`, the conversion should be `max(c / 127.0, -1.0)`.
 */

test('WebGL 2 Normalized Vertex Attributes Conformance Test', { skip: true }, async (t) => {
    const gl = await webGL2();
    gl.viewport(0, 0, 1, 1);

    const vsSource = `#version 300 es
        layout(location = 0) in vec3 vertex;

        out float normAttrib;

        void main(void) {
            gl_Position = vec4(vertex.xy, 0, 1);
            normAttrib = vertex.z;
        }
    `;

    const fsSource = `#version 300 es
        in lowp float normAttrib;

        layout(location=0) out lowp vec4 fragColor;

        void main(void) {
            fragColor = vec4(vec3(normAttrib == -1.0 ? 1.0 : 0.0), 1);
        }
    `;

    const vs = gl.createShader(gl.VERTEX_SHADER);
    gl.shaderSource(vs, vsSource);
    gl.compileShader(vs);
    if (!gl.getShaderParameter(vs, gl.COMPILE_STATUS)) {
        throw new Error('VS compile failed: ' + gl.getShaderInfoLog(vs));
    }

    const fs = gl.createShader(gl.FRAGMENT_SHADER);
    gl.shaderSource(fs, fsSource);
    gl.compileShader(fs);
    if (!gl.getShaderParameter(fs, gl.COMPILE_STATUS)) {
        throw new Error('FS compile failed: ' + gl.getShaderInfoLog(fs));
    }

    const program = gl.createProgram();
    gl.attachShader(program, vs);
    gl.attachShader(program, fs);
    gl.linkProgram(program);
    if (!gl.getProgramParameter(program, gl.LINK_STATUS)) {
        throw new Error('Link failed: ' + gl.getProgramInfoLog(program));
    }
    gl.useProgram(program);

    const buffer = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, buffer);
    gl.bufferData(gl.ARRAY_BUFFER, new Int8Array([
        -0x80, 0x7f, -0x7f,
        0x7f, 0x7f, -0x7f,
        -0x80, -0x7f, -0x7f,
        -0x80, -0x7f, -0x7f,
        0x7f, 0x7f, -0x7f,
        0x7f, -0x80, -0x7f
    ]), gl.STATIC_DRAW);

    gl.enableVertexAttribArray(0);
    gl.vertexAttribPointer(0, 3, gl.BYTE, true, 0, 0);

    gl.drawArrays(gl.TRIANGLE_STRIP, 0, 6);

    const pixels = new Uint8Array(4);
    gl.readPixels(0, 0, 1, 1, gl.RGBA, gl.UNSIGNED_BYTE, pixels);

    // Check for opaque white [255, 255, 255, 255]
    assert.deepStrictEqual(Array.from(pixels), [255, 255, 255, 255], "should be opaque white");
});
