import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('refactor_safety_fragment_color: verify basic rendering still works after refactor', async () => {
  const gl = await webGL2();
  try {
    gl.viewport(0, 0, 1, 1);
    
    const vs = gl.createShader(gl.VERTEX_SHADER);
    gl.shaderSource(vs, `#version 300 es
      void main() { gl_Position = vec4(0.0, 0.0, 0.0, 1.0); gl_PointSize = 1.0; }`);
    gl.compileShader(vs);
    
    const fs = gl.createShader(gl.FRAGMENT_SHADER);
    gl.shaderSource(fs, `#version 300 es
      precision highp float;
      out vec4 fragColor;
      void main() { fragColor = vec4(1.0, 0.5, 0.25, 1.0); }`);
    gl.compileShader(fs);
    
    const prog = gl.createProgram();
    gl.attachShader(prog, vs);
    gl.attachShader(prog, fs);
    gl.linkProgram(prog);
    gl.useProgram(prog);
    
    gl.clearColor(0, 0, 0, 1);
    gl.clear(gl.COLOR_BUFFER_BIT);
    gl.drawArrays(gl.POINTS, 0, 1);
    
    const pixels = new Uint8Array(4);
    gl.readPixels(0, 0, 1, 1, gl.RGBA, gl.UNSIGNED_BYTE, pixels);
    
    assert.strictEqual(pixels[0], 255, 'Red channel');
    // 0.5 * 255 = 127.5. Renderers might round to 127 or 128.
    assert.ok(pixels[1] === 127 || pixels[1] === 128, `Green channel: ${pixels[1]}`);
    // 0.25 * 255 = 63.75. Renderers might round to 63 or 64.
    assert.ok(pixels[2] === 63 || pixels[2] === 64, `Blue channel: ${pixels[2]}`);
    assert.strictEqual(pixels[3], 255, 'Alpha channel');
  } finally {
    gl.destroy();
  }
});
