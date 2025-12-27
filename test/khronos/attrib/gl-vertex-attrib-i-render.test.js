import test from 'node:test';
import assert from 'node:assert';
import { webGL2 } from '../../../index.js';

test('WebGL2 vertexAttribI* rendering', async (t) => {
  const gl = await webGL2();
  gl.viewport(0, 0, 640, 480);

  const vsSource = `#version 300 es
    layout(location = 0) in ivec4 a_ivec4;
    layout(location = 1) in uvec4 a_uvec4;
    flat out ivec4 v_ivec4;
    flat out uvec4 v_uvec4;
    void main() {
        v_ivec4 = a_ivec4;
        v_uvec4 = a_uvec4;
        gl_Position = vec4(0.0, 0.0, 0.0, 1.0);
    }`;

  const fsSource = `#version 300 es
    precision highp float;
    flat in ivec4 v_ivec4;
    flat in uvec4 v_uvec4;
    out vec4 fragColor;
    void main() {
        bool ok = true;
        if (v_ivec4 != ivec4(-1, 2, -3, 4)) ok = false;
        if (v_uvec4 != uvec4(1u, 2u, 3u, 4u)) ok = false;
        
        if (ok) {
            fragColor = vec4(0.0, 1.0, 0.0, 1.0); // Green
        } else {
            fragColor = vec4(1.0, 0.0, 0.0, 1.0); // Red
        }
    }`;

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

  // Test 1: vertexAttribI4i and vertexAttribI4ui (constant attributes)
  await t.test('constant attributes', () => {
    gl.vertexAttribI4i(0, -1, 2, -3, 4);
    gl.vertexAttribI4ui(1, 1, 2, 3, 4);

    gl.clearColor(0, 0, 0, 1);
    gl.clear(gl.COLOR_BUFFER_BIT);
    gl.drawArrays(gl.POINTS, 0, 1);

    const pixels = new Uint8Array(4);
    gl.readPixels(320, 240, 1, 1, gl.RGBA, gl.UNSIGNED_BYTE, pixels);
    console.log('Test 1 Pixels:', Array.from(pixels));
    assert.deepStrictEqual(Array.from(pixels), [0, 255, 0, 255], 'Should be green');
  });

  // Test 2: vertexAttribIPointer with i32
  await t.test('vertexAttribIPointer i32', () => {
    const buffer = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, buffer);
    const data = new Int32Array([-1, 2, -3, 4]);
    gl.bufferData(gl.ARRAY_BUFFER, data, gl.STATIC_DRAW);

    gl.enableVertexAttribArray(0);
    gl.vertexAttribIPointer(0, 4, gl.INT, 0, 0);

    gl.vertexAttribI4ui(1, 1, 2, 3, 4); // Keep uvec4 constant

    gl.clear(gl.COLOR_BUFFER_BIT);
    gl.drawArrays(gl.POINTS, 0, 1);

    const pixels = new Uint8Array(4);
    gl.readPixels(320, 240, 1, 1, gl.RGBA, gl.UNSIGNED_BYTE, pixels);
    assert.deepStrictEqual(Array.from(pixels), [0, 255, 0, 255], 'Should be green');
  });

  // Test 3: vertexAttribIPointer with u32
  await t.test('vertexAttribIPointer u32', () => {
    const buffer = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, buffer);
    const data = new Uint32Array([1, 2, 3, 4]);
    gl.bufferData(gl.ARRAY_BUFFER, data, gl.STATIC_DRAW);

    gl.enableVertexAttribArray(1);
    gl.vertexAttribIPointer(1, 4, gl.UNSIGNED_INT, 0, 0);

    gl.vertexAttribI4i(0, -1, 2, -3, 4); // Keep ivec4 constant

    gl.clear(gl.COLOR_BUFFER_BIT);
    gl.drawArrays(gl.POINTS, 0, 1);

    const pixels = new Uint8Array(4);
    gl.readPixels(320, 240, 1, 1, gl.RGBA, gl.UNSIGNED_BYTE, pixels);
    assert.deepStrictEqual(Array.from(pixels), [0, 255, 0, 255], 'Should be green');
  });

  // Test 4: vertexAttribIPointer with i8/u8/i16/u16
  await t.test('vertexAttribIPointer small types', () => {
    // Test i8
    const bufI8 = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, bufI8);
    gl.bufferData(gl.ARRAY_BUFFER, new Int8Array([-1, 2, -3, 4]), gl.STATIC_DRAW);
    gl.enableVertexAttribArray(0);
    gl.vertexAttribIPointer(0, 4, gl.BYTE, 0, 0);

    // Test u16
    const bufU16 = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, bufU16);
    gl.bufferData(gl.ARRAY_BUFFER, new Uint16Array([1, 2, 3, 4]), gl.STATIC_DRAW);
    gl.enableVertexAttribArray(1);
    gl.vertexAttribIPointer(1, 4, gl.UNSIGNED_SHORT, 0, 0);

    gl.clear(gl.COLOR_BUFFER_BIT);
    gl.drawArrays(gl.POINTS, 0, 1);

    const pixels = new Uint8Array(4);
    gl.readPixels(320, 240, 1, 1, gl.RGBA, gl.UNSIGNED_BYTE, pixels);
    assert.deepStrictEqual(Array.from(pixels), [0, 255, 0, 255], 'Should be green');
  });
});
