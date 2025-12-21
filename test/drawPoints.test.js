import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('drawArrays(POINTS) draws to default framebuffer', async () => {
  const gl = await webGL2();
  try {
    // 1. Set up a buffer with one point at (0,0) in NDC
    const buf = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, buf);
    const data = new Float32Array([0.0, 0.0, 0.0, 1.0]);
    gl.bufferData(gl.ARRAY_BUFFER, data, gl.STATIC_DRAW);

    // 2. Set up attribute 0
    gl.enableVertexAttribArray(0);
    gl.vertexAttribPointer(0, 4, gl.FLOAT, false, 0, 0);

    // 3. Create and use a dummy program (required by our implementation)
    const vs = gl.createShader(gl.VERTEX_SHADER);
    gl.shaderSource(vs, `#version 300 es
layout(location = 0) in vec4 pos;
uniform float off_x;
uniform float off_y;
void main() {
    gl_Position = pos + vec4(off_x, off_y, 0.0, 0.0);
}`);
    gl.compileShader(vs);
    const fs = gl.createShader(gl.FRAGMENT_SHADER);
    gl.shaderSource(fs, '#version 300 es\nprecision highp float;\nout vec4 color;\nvoid main() { color = vec4(1,1,1,1); }');
    gl.compileShader(fs);
    const prog = gl.createProgram();
    gl.attachShader(prog, vs);
    gl.attachShader(prog, fs);
    gl.linkProgram(prog);
    gl.useProgram(prog);

    // Set uniform offsets
    const locX = gl.getUniformLocation(prog, 'off_x');
    const locY = gl.getUniformLocation(prog, 'off_y');
    gl.uniform1f(locX, 0.1);
    gl.uniform1f(locY, 0.1);

    // 4. Clear to black
    gl.clearColor(0, 0, 0, 1);
    gl.clear(gl.COLOR_BUFFER_BIT);

    // 5. Draw the point
    gl.drawArrays(gl.POINTS, 0, 1);

    // 6. Read back the shifted pixel
    // (0,0) NDC + (0.1, 0.1) = (0.1, 0.1) NDC
    // (0.1, 0.1) NDC -> (320 + 32, 240 + 24) = (352, 264)
    const pixels = new Uint8Array(4);
    gl.readPixels(352, 264, 1, 1, gl.RGBA, gl.UNSIGNED_BYTE, pixels);

    assert.strictEqual(pixels[0], 255, 'Red should be 255');
    assert.strictEqual(pixels[1], 255, 'Green should be 255');
    assert.strictEqual(pixels[2], 255, 'Blue should be 255');
    assert.strictEqual(pixels[3], 255, 'Alpha should be 255');

  } finally {
    gl.destroy();
  }
});
