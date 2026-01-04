
import test from 'node:test';
import assert from 'node:assert';
import { webGL2 } from '../index.js';

test('Integer Attribute Values Probe', async (t) => {
    const gl = await webGL2();
    try {
        gl.viewport(0, 0, 1, 1);

        const vsSource = `#version 300 es
    in ivec4 a_val;
    flat out ivec4 v_val;
    void main() {
        v_val = a_val;
        gl_Position = vec4(0.0, 0.0, 0.0, 1.0);
        gl_PointSize = 64.0;
    }`;

        const fsSource = `#version 300 es
    precision highp float;
    precision highp int;
    flat in ivec4 v_val;
    out vec4 outColor;
    void main() {
        // Map integer values 0-255 directly to 0.0-1.0
        outColor = vec4(
            float(v_val.x) / 255.0,
            float(v_val.y) / 255.0,
            float(v_val.z) / 255.0,
            float(v_val.w) / 255.0
        );
    }`;

        const program = gl.createProgram();
        const vs = gl.createShader(gl.VERTEX_SHADER);
        gl.shaderSource(vs, vsSource);
        gl.compileShader(vs);
        if (!gl.getShaderParameter(vs, gl.COMPILE_STATUS)) {
            console.error(gl.getShaderInfoLog(vs));
            assert.fail('Vertex shader compilation failed');
        }

        const fs = gl.createShader(gl.FRAGMENT_SHADER);
        gl.shaderSource(fs, fsSource);
        gl.compileShader(fs);
        if (!gl.getShaderParameter(fs, gl.COMPILE_STATUS)) {
            console.error(gl.getShaderInfoLog(fs));
            assert.fail('Fragment shader compilation failed');
        }

        gl.attachShader(program, vs);
        gl.attachShader(program, fs);
        gl.linkProgram(program);
        if (!gl.getProgramParameter(program, gl.LINK_STATUS)) {
            console.error(gl.getProgramInfoLog(program));
            assert.fail('Program linking failed');
        }
        gl.useProgram(program);

        // Test values: 10, 20, 30, 40
        // We expect to read back roughly these values in the color channels
        const data = new Int32Array([10, 20, 30, 40]);

        const buffer = gl.createBuffer();
        gl.bindBuffer(gl.ARRAY_BUFFER, buffer);
        gl.bufferData(gl.ARRAY_BUFFER, data, gl.STATIC_DRAW);

        const loc = gl.getAttribLocation(program, 'a_val');
        gl.enableVertexAttribArray(loc);
        gl.vertexAttribIPointer(loc, 4, gl.INT, 0, 0);

        gl.drawArrays(gl.POINTS, 0, 1);

        const pixels = new Uint8Array(4);
        gl.readPixels(0, 0, 1, 1, gl.RGBA, gl.UNSIGNED_BYTE, pixels);

        // Allow for small rounding errors
        assert.ok(Math.abs(pixels[0] - 10) <= 1, `Red channel (x) should be ~10, got ${pixels[0]}`);
        assert.ok(Math.abs(pixels[1] - 20) <= 1, `Green channel (y) should be ~20, got ${pixels[1]}`);
        assert.ok(Math.abs(pixels[2] - 30) <= 1, `Blue channel (z) should be ~30, got ${pixels[2]}`);
        assert.ok(Math.abs(pixels[3] - 40) <= 1, `Alpha channel (w) should be ~40, got ${pixels[3]}`);

        // --- Test 2: Unsigned Int Attributes ---
        const vsSourceU = `#version 300 es
    in uvec4 a_val;
    flat out uvec4 v_val;
    void main() {
        v_val = a_val;
        gl_Position = vec4(0.0, 0.0, 0.0, 1.0);
        gl_PointSize = 64.0;
    }`;

        const fsSourceU = `#version 300 es
    precision highp float;
    precision highp int;
    flat in uvec4 v_val;
    out vec4 outColor;
    void main() {
        outColor = vec4(
            float(v_val.x) / 255.0,
            float(v_val.y) / 255.0,
            float(v_val.z) / 255.0,
            float(v_val.w) / 255.0
        );
    }`;

        const programU = gl.createProgram();
        const vsU = gl.createShader(gl.VERTEX_SHADER);
        gl.shaderSource(vsU, vsSourceU);
        gl.compileShader(vsU);
        const fsU = gl.createShader(gl.FRAGMENT_SHADER);
        gl.shaderSource(fsU, fsSourceU);
        gl.compileShader(fsU);
        gl.attachShader(programU, vsU);
        gl.attachShader(programU, fsU);
        gl.linkProgram(programU);
        gl.useProgram(programU);

        const dataU = new Uint32Array([50, 60, 70, 80]);
        const bufferU = gl.createBuffer();
        gl.bindBuffer(gl.ARRAY_BUFFER, bufferU);
        gl.bufferData(gl.ARRAY_BUFFER, dataU, gl.STATIC_DRAW);

        const locU = gl.getAttribLocation(programU, 'a_val');
        gl.enableVertexAttribArray(locU);
        gl.vertexAttribIPointer(locU, 4, gl.UNSIGNED_INT, 0, 0);

        gl.drawArrays(gl.POINTS, 0, 1);
        gl.readPixels(0, 0, 1, 1, gl.RGBA, gl.UNSIGNED_BYTE, pixels);
        assert.ok(Math.abs(pixels[0] - 50) <= 1, `Red (x) ~50, got ${pixels[0]}`);
        assert.ok(Math.abs(pixels[1] - 60) <= 1, `Green (y) ~60, got ${pixels[1]}`);
        assert.ok(Math.abs(pixels[2] - 70) <= 1, `Blue (z) ~70, got ${pixels[2]}`);
        assert.ok(Math.abs(pixels[3] - 80) <= 1, `Alpha (w) ~80, got ${pixels[3]}`);

        // --- Test 3: Constant Attributes ---
        gl.disableVertexAttribArray(locU);
        gl.vertexAttribI4ui(locU, 90, 100, 110, 120);
        gl.drawArrays(gl.POINTS, 0, 1);
        gl.readPixels(0, 0, 1, 1, gl.RGBA, gl.UNSIGNED_BYTE, pixels);
        assert.ok(Math.abs(pixels[0] - 90) <= 1, `Red (x) ~90, got ${pixels[0]}`);
        assert.ok(Math.abs(pixels[1] - 100) <= 1, `Green (y) ~100, got ${pixels[1]}`);
        assert.ok(Math.abs(pixels[2] - 110) <= 1, `Blue (z) ~110, got ${pixels[2]}`);
        assert.ok(Math.abs(pixels[3] - 120) <= 1, `Alpha (w) ~120, got ${pixels[3]}`);

        // --- Test 4: Triangles (Interpolation) ---
        // Use the first program (ivec4)
        gl.useProgram(program);
        const locT = gl.getAttribLocation(program, 'a_val');
        gl.enableVertexAttribArray(locT);

        // 3 vertices, same value for all to test flat shading (or just data passing)
        // V0: 10,20,30,40
        // V1: 10,20,30,40
        // V2: 10,20,30,40
        const dataT = new Int32Array([
            10, 20, 30, 40,
            10, 20, 30, 40,
            10, 20, 30, 40
        ]);
        const bufferT = gl.createBuffer();
        gl.bindBuffer(gl.ARRAY_BUFFER, bufferT);
        gl.bufferData(gl.ARRAY_BUFFER, dataT, gl.STATIC_DRAW);
        gl.vertexAttribIPointer(locT, 4, gl.INT, 0, 0);

        // We need a new shader that draws a triangle covering the pixel
        const vsSourceT = `#version 300 es
    in vec4 a_pos;
    in ivec4 a_val;
    flat out ivec4 v_val;
    void main() {
        v_val = a_val;
        gl_Position = a_pos;
    }`;
        const vsT = gl.createShader(gl.VERTEX_SHADER);
        gl.shaderSource(vsT, vsSourceT);
        gl.compileShader(vsT);
        const programT = gl.createProgram();
        gl.attachShader(programT, vsT);
        gl.attachShader(programT, fs); // Reuse FS
        gl.bindAttribLocation(programT, 0, 'a_pos');
        gl.bindAttribLocation(programT, 1, 'a_val');
        gl.linkProgram(programT);
        if (!gl.getProgramParameter(programT, gl.LINK_STATUS)) {
            console.error(gl.getProgramInfoLog(programT));
            throw new Error('Program link failed');
        }
        gl.useProgram(programT);

        const locPos = gl.getAttribLocation(programT, 'a_pos');
        const posData = new Float32Array([
            -1.0, -1.0, 0.0, 1.0,
            3.0, -1.0, 0.0, 1.0,
            -1.0, 3.0, 0.0, 1.0
        ]);
        const posBuffer = gl.createBuffer();
        gl.bindBuffer(gl.ARRAY_BUFFER, posBuffer);
        gl.bufferData(gl.ARRAY_BUFFER, posData, gl.STATIC_DRAW);
        gl.enableVertexAttribArray(locPos);
        gl.vertexAttribPointer(locPos, 4, gl.FLOAT, false, 0, 0);

        const locT2 = gl.getAttribLocation(programT, 'a_val');
        if (locT2 === -1) throw new Error('a_val location not found');
        gl.enableVertexAttribArray(locT2);
        gl.bindBuffer(gl.ARRAY_BUFFER, bufferT);
        gl.vertexAttribIPointer(locT2, 4, gl.INT, 0, 0);

        gl.drawArrays(gl.TRIANGLES, 0, 3);
        gl.readPixels(0, 0, 1, 1, gl.RGBA, gl.UNSIGNED_BYTE, pixels);
        assert.ok(Math.abs(pixels[0] - 10) <= 1, `Red (x) ~10, got ${pixels[0]}`);
        assert.ok(Math.abs(pixels[1] - 20) <= 1, `Green (y) ~20, got ${pixels[1]}`);
        assert.ok(Math.abs(pixels[2] - 30) <= 1, `Blue (z) ~30, got ${pixels[2]}`);
        assert.ok(Math.abs(pixels[3] - 40) <= 1, `Alpha (w) ~40, got ${pixels[3]}`);

    } finally {
        gl.destroy();
    }
});
