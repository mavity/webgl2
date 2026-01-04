import test from 'node:test';
import assert from 'node:assert';
import { webGL2 } from '../index.js';

test('Granular Debugging', async (t) => {
  const gl = await webGL2();
  gl.viewport(0, 0, 10, 10);

  // Helper to compile program
  function createProgram(vsSrc, fsSrc) {
    const vs = gl.createShader(gl.VERTEX_SHADER);
    gl.shaderSource(vs, vsSrc);
    gl.compileShader(vs);
    if (!gl.getShaderParameter(vs, gl.COMPILE_STATUS)) throw new Error(gl.getShaderInfoLog(vs));

    const fs = gl.createShader(gl.FRAGMENT_SHADER);
    gl.shaderSource(fs, fsSrc);
    gl.compileShader(fs);
    if (!gl.getShaderParameter(fs, gl.COMPILE_STATUS)) throw new Error(gl.getShaderInfoLog(fs));

    const p = gl.createProgram();
    gl.attachShader(p, vs);
    gl.attachShader(p, fs);
    gl.linkProgram(p);
    if (!gl.getProgramParameter(p, gl.LINK_STATUS)) throw new Error(gl.getProgramInfoLog(p));
    return p;
  }

  // Test 1: Simple Color Output (No inputs)
  await t.test('Simple Color Output', () => {
    const vs = `#version 300 es
      void main() {
        gl_Position = vec4(0.0, 0.0, 0.0, 1.0);
        gl_PointSize = 10.0;
      }`;
    const fs = `#version 300 es
      precision highp float;
      out vec4 fragColor;
      void main() {
        fragColor = vec4(0.0, 1.0, 0.0, 1.0);
      }`;
    const p = createProgram(vs, fs);
    gl.useProgram(p);
    gl.clearColor(0, 0, 0, 0);
    gl.clear(gl.COLOR_BUFFER_BIT);
    gl.drawArrays(gl.POINTS, 0, 1);

    const pixels = new Uint8Array(4);
    gl.readPixels(5, 5, 1, 1, gl.RGBA, gl.UNSIGNED_BYTE, pixels);
    assert.deepStrictEqual(Array.from(pixels), [0, 255, 0, 255]);
  });

  // Test 2: Float Varying Passthrough
  await t.test('Float Varying', () => {
    const vs = `#version 300 es
      out float v_val;
      void main() {
        v_val = 0.5;
        gl_Position = vec4(0.0, 0.0, 0.0, 1.0);
        gl_PointSize = 10.0;
      }`;
    const fs = `#version 300 es
      precision highp float;
      in float v_val;
      out vec4 fragColor;
      void main() {
        fragColor = vec4(v_val, v_val, v_val, 1.0);
      }`;
    const p = createProgram(vs, fs);
    gl.useProgram(p);
    gl.clear(gl.COLOR_BUFFER_BIT);
    gl.drawArrays(gl.POINTS, 0, 1);

    const pixels = new Uint8Array(4);
    gl.readPixels(5, 5, 1, 1, gl.RGBA, gl.UNSIGNED_BYTE, pixels);
    // 0.5 * 255 = 127.5 -> 127 or 128
    const val = pixels[0];
    assert.ok(val >= 127 && val <= 128, `Expected ~127, got ${val}`);
    assert.strictEqual(pixels[3], 255);
  });

  // Test 3: Flat Int Varying
  await t.test('Flat Int Varying', () => {
    const vs = `#version 300 es
      flat out int v_val;
      void main() {
        v_val = 42;
        gl_Position = vec4(0.0, 0.0, 0.0, 1.0);
        gl_PointSize = 10.0;
      }`;
    const fs = `#version 300 es
      precision highp float;
      flat in int v_val;
      out vec4 fragColor;
      void main() {
        if (v_val == 42) fragColor = vec4(0.0, 1.0, 0.0, 1.0);
        else fragColor = vec4(1.0, 0.0, 0.0, 1.0);
      }`;
    const p = createProgram(vs, fs);
    gl.useProgram(p);
    gl.clear(gl.COLOR_BUFFER_BIT);
    gl.drawArrays(gl.POINTS, 0, 1);

    const pixels = new Uint8Array(4);
    gl.readPixels(5, 5, 1, 1, gl.RGBA, gl.UNSIGNED_BYTE, pixels);
    assert.deepStrictEqual(Array.from(pixels), [0, 255, 0, 255]);
  });

  // Test 4: Flat IVec4 Varying
  await t.test('Flat IVec4 Varying', () => {
    const vs = `#version 300 es
      flat out ivec4 v_val;
      void main() {
        v_val = ivec4(-1, 2, -3, 4);
        gl_Position = vec4(0.0, 0.0, 0.0, 1.0);
        gl_PointSize = 10.0;
      }`;
    const fs = `#version 300 es
      precision highp float;
      flat in ivec4 v_val;
      out vec4 fragColor;
      void main() {
        if (v_val == ivec4(-1, 2, -3, 4)) fragColor = vec4(0.0, 1.0, 0.0, 1.0);
        else fragColor = vec4(1.0, 0.0, 0.0, 1.0);
      }`;
    const p = createProgram(vs, fs);
    gl.useProgram(p);
    gl.clear(gl.COLOR_BUFFER_BIT);
    gl.drawArrays(gl.POINTS, 0, 1);

    const pixels = new Uint8Array(4);
    gl.readPixels(5, 5, 1, 1, gl.RGBA, gl.UNSIGNED_BYTE, pixels);
    assert.deepStrictEqual(Array.from(pixels), [0, 255, 0, 255]);
  });
});
