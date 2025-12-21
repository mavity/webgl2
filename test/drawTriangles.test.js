import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('drawArrays(TRIANGLES) draws to default framebuffer', async () => {
  const gl = await webGL2();
  try {
    // 1. Set up a buffer with one triangle
    const buf = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, buf);
    const data = new Float32Array([
      -0.5, -0.5, 0.0, 1.0,
       0.5, -0.5, 0.0, 1.0,
       0.0,  0.5, 0.0, 1.0,
    ]);
    gl.bufferData(gl.ARRAY_BUFFER, data, gl.STATIC_DRAW);

    // 2. Set up attribute 0
    gl.enableVertexAttribArray(0);
    gl.vertexAttribPointer(0, 4, gl.FLOAT, false, 0, 0);

    // 3. Create and use a dummy program
    const vs = gl.createShader(gl.VERTEX_SHADER);
    gl.shaderSource(vs, '#version 300 es\nvoid main() { gl_Position = vec4(0,0,0,1); }');
    gl.compileShader(vs);
    const fs = gl.createShader(gl.FRAGMENT_SHADER);
    gl.shaderSource(fs, '#version 300 es\nprecision highp float;\nout vec4 color;\nvoid main() { color = vec4(1,1,1,1); }');
    gl.compileShader(fs);
    const prog = gl.createProgram();
    gl.attachShader(prog, vs);
    gl.attachShader(prog, fs);
    gl.linkProgram(prog);
    gl.useProgram(prog);

    // 4. Clear to black
    gl.clearColor(0, 0, 0, 1);
    gl.clear(gl.COLOR_BUFFER_BIT);

    // 5. Draw the triangle
    gl.drawArrays(gl.TRIANGLES, 0, 3);

    // 6. Read back a pixel inside the triangle (center: 0,0 NDC -> 320, 240 screen)
    const pixels = new Uint8Array(4);
    gl.readPixels(320, 240, 1, 1, gl.RGBA, gl.UNSIGNED_BYTE, pixels);

    assert.strictEqual(pixels[0], 255, 'Red should be 255');
    assert.strictEqual(pixels[1], 255, 'Green should be 255');
    assert.strictEqual(pixels[2], 255, 'Blue should be 255');
    assert.strictEqual(pixels[3], 255, 'Alpha should be 255');

    // 7. Read back a pixel outside the triangle (top-left: -1,1 NDC -> 0, 480 screen)
    gl.readPixels(0, 479, 1, 1, gl.RGBA, gl.UNSIGNED_BYTE, pixels);
    assert.strictEqual(pixels[0], 0, 'Red should be 0 outside');

  } finally {
    gl.destroy();
  }
});
