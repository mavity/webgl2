import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('sampler2D specialized sampler works', async () => {
  const gl = await webGL2();
  try {
    const vs = `#version 300 es
layout(location = 0) in vec2 pos;
uniform float unitest;
out vec2 v_uv;
void main() {
  v_uv = pos * 0.5 + 0.5;
  gl_Position = vec4(pos + unitest, 0.0, 1.0);
}`;

    const fs = `#version 300 es
precision highp float;
uniform highp sampler2D tex;
in vec2 v_uv;
out vec4 color;
void main() {
  color = texture(tex, v_uv);
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

    // Create 2D texture 2x2
    const tex = gl.createTexture();
    gl.bindTexture(gl.TEXTURE_2D, tex);
    const data = new Uint8Array([
      255, 0, 0, 255, 0, 255, 0, 255,
      0, 0, 255, 255, 255, 255, 0, 255,
    ]);
    gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA8, 2, 2, 0, gl.RGBA, gl.UNSIGNED_BYTE, data);

    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.NEAREST);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.NEAREST);

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

    // Bottom-left (Red)
    gl.readPixels(0, 0, 1, 1, gl.RGBA, gl.UNSIGNED_BYTE, out);
    assert.deepStrictEqual(Array.from(out), [255, 0, 0, 255]);

    // Top-right (Yellow: 255, 255, 0)
    gl.readPixels(gl.drawingBufferWidth - 1, gl.drawingBufferHeight - 1, 1, 1, gl.RGBA, gl.UNSIGNED_BYTE, out);
    assert.deepStrictEqual(Array.from(out), [255, 255, 0, 255]);

  } finally {
    gl.destroy();
  }
});
