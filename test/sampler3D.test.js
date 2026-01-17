import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('sampler3D specialized sampler works', async () => {
  const gl = await webGL2();
  try {
    const vs = `#version 300 es
      layout(location = 0) in vec2 pos;
      out vec2 v_uv;
      void main() {
        v_uv = pos * 0.5 + 0.5;
        gl_Position = vec4(pos, 0.0, 1.0);
      }`;

    // Sample from 3D texture using v_uv for X,Y and a constant for Z
    const fs = `#version 300 es
      precision highp float;
      uniform sampler3D tex;
      in vec2 v_uv;
      out vec4 color;
      void main() {
        // Red = layer 0, Green = layer 1
        // We pick Z based on X to see both in one draw
        float z = v_uv.x > 0.5 ? 0.75 : 0.25;
        color = texture(tex, vec3(v_uv.x, v_uv.y, z));
      }`;

    const program = gl.createProgram();
    const vShader = gl.createShader(gl.VERTEX_SHADER);
    gl.shaderSource(vShader, vs);
    gl.compileShader(vShader);
    const fShader = gl.createShader(gl.FRAGMENT_SHADER);
    gl.shaderSource(fShader, fs);
    gl.compileShader(fShader);
    gl.attachShader(program, vShader);
    gl.attachShader(program, fShader);
    gl.linkProgram(program);
    gl.useProgram(program);

    // Create 3D texture 2x2x2
    const tex = gl.createTexture();
    gl.bindTexture(gl.TEXTURE_3D, tex);
    const data = new Uint8Array([
      // Layer 0: all red
      255, 0, 0, 255, 255, 0, 0, 255,
      255, 0, 0, 255, 255, 0, 0, 255,
      // Layer 1: all green
      0, 255, 0, 255, 0, 255, 0, 255,
      0, 255, 0, 255, 0, 255, 0, 255,
    ]);
    gl.texImage3D(gl.TEXTURE_3D, 0, gl.RGBA8, 2, 2, 2, 0, gl.RGBA, gl.UNSIGNED_BYTE, data);

    // Set parameters to ensure sampling works as expected
    gl.texParameteri(gl.TEXTURE_3D, gl.TEXTURE_MIN_FILTER, gl.NEAREST);
    gl.texParameteri(gl.TEXTURE_3D, gl.TEXTURE_MAG_FILTER, gl.NEAREST);
    gl.texParameteri(gl.TEXTURE_3D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_3D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_3D, gl.TEXTURE_WRAP_R, gl.CLAMP_TO_EDGE);

    // Set uniform
    const loc = gl.getUniformLocation(program, 'tex');
    gl.uniform1i(loc, 0);

    // Draw quad
    const buf = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, buf);
    gl.bufferData(gl.ARRAY_BUFFER, new Float32Array([-1, -1, 1, -1, -1, 1, 1, 1]), gl.STATIC_DRAW);
    gl.enableVertexAttribArray(0);
    gl.vertexAttribPointer(0, 2, gl.FLOAT, false, 0, 0);

    gl.drawArrays(gl.TRIANGLE_STRIP, 0, 4);

    // Check pixels
    const out = new Uint8Array(4);

    // Left side should be Layer 0 (Red)
    gl.readPixels(0, 0, 1, 1, gl.RGBA, gl.UNSIGNED_BYTE, out);
    assert.deepStrictEqual(Array.from(out), [255, 0, 0, 255]);

    // Right side should be Layer 1 (Green)
    gl.readPixels(gl.drawingBufferWidth - 1, 0, 1, 1, gl.RGBA, gl.UNSIGNED_BYTE, out);
    assert.deepStrictEqual(Array.from(out), [0, 255, 0, 255]);

  } finally {
    gl.destroy();
  }
});
