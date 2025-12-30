import test from 'node:test';
import assert from 'node:assert';
import { webGL2 } from '../index.js';

// This test suite verifies that vertex attributes are correctly interpreted
// as their declared types (int, uint, float) in the shader.

test('Shader Input Types', async (t) => {
    const gl = await webGL2({ debug: 'shaders' });
    gl.viewport(0, 0, 1, 1);

    // Helper to compile program
    function createProgram(vsSrc, fsSrc) {
        const vs = gl.createShader(gl.VERTEX_SHADER);
        gl.shaderSource(vs, vsSrc);
        gl.compileShader(vs);
        if (!gl.getShaderParameter(vs, gl.COMPILE_STATUS)) {
            throw new Error('VS: ' + gl.getShaderInfoLog(vs));
        }
        const fs = gl.createShader(gl.FRAGMENT_SHADER);
        gl.shaderSource(fs, fsSrc);
        gl.compileShader(fs);
        if (!gl.getShaderParameter(fs, gl.COMPILE_STATUS)) {
            throw new Error('FS: ' + gl.getShaderInfoLog(fs));
        }
        const prog = gl.createProgram();
        gl.attachShader(prog, vs);
        gl.attachShader(prog, fs);
        gl.linkProgram(prog);
        if (!gl.getProgramParameter(prog, gl.LINK_STATUS)) {
            throw new Error('Link: ' + gl.getProgramInfoLog(prog));
        }
        return prog;
    }

    // Test 1: Integer Attribute (int)
    // We pass -1 (0xFFFFFFFF). If interpreted as float, it is NaN.
    await t.test('int attribute', () => {
        const vs = `#version 300 es
        layout(location=0) in int a_val;
        flat out int v_val;
        void main() { v_val = a_val; gl_Position = vec4(0,0,0,1); gl_PointSize = 1.0; }`;
        const fs = `#version 300 es
        precision highp float;
        flat in int v_val;
        out vec4 color;
        void main() {
            if (v_val == -1) color = vec4(0,1,0,1); // Green
            else color = vec4(1,0,0,1); // Red
        }`;
        const prog = createProgram(vs, fs);
        gl.useProgram(prog);
        gl.vertexAttribI4i(0, -1, 0, 0, 0);
        gl.clearColor(0,0,0,0);
        gl.clear(gl.COLOR_BUFFER_BIT);
        gl.drawArrays(gl.POINTS, 0, 1);
        const pixels = new Uint8Array(4);
        gl.readPixels(0,0,1,1, gl.RGBA, gl.UNSIGNED_BYTE, pixels);
        assert.deepStrictEqual(Array.from(pixels), [0, 255, 0, 255], 'Should read -1 correctly');
    });

    // Test 2: Unsigned Integer Attribute (uint)
    // We pass 0xFFFFFFFF (MAX_UINT).
    await t.test('uint attribute', () => {
        const vs = `#version 300 es
        layout(location=0) in uint a_val;
        flat out uint v_val;
        void main() { v_val = a_val; gl_Position = vec4(0,0,0,1); gl_PointSize = 1.0; }`;
        const fs = `#version 300 es
        precision highp float;
        flat in uint v_val;
        out vec4 color;
        void main() {
            if (v_val == 4294967295u) color = vec4(0,1,0,1);
            else color = vec4(1,0,0,1);
        }`;
        const prog = createProgram(vs, fs);
        gl.useProgram(prog);
        gl.vertexAttribI4ui(0, 0xFFFFFFFF, 0, 0, 0);
        gl.clearColor(0,0,0,0);
        gl.clear(gl.COLOR_BUFFER_BIT);
        gl.drawArrays(gl.POINTS, 0, 1);
        const pixels = new Uint8Array(4);
        gl.readPixels(0,0,1,1, gl.RGBA, gl.UNSIGNED_BYTE, pixels);
        assert.deepStrictEqual(Array.from(pixels), [0, 255, 0, 255], 'Should read MAX_UINT correctly');
    });
});
