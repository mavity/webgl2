import test from 'node:test';
import assert from 'node:assert';
import { webGL2 } from '../index.js';

// Verify fragment shader stores appear in the shared private memory region
// (private_ptr default is 0x4000 in RasterPipeline::default())
test('Private memory roundtrip (fragment store visible to host)', async (t) => {
  const gl = await webGL2();
  try {
    gl.viewport(0, 0, 64, 64);

    const vs = `#version 300 es
    void main(){ gl_Position = vec4(0.0, 0.0, 0.0, 1.0); gl_PointSize = 1.0; }`;

    // Use a distinct non-zero color so memory writes are obvious
    const fs = `#version 300 es
    precision highp float;
    out vec4 fragColor;
    void main(){ fragColor = vec4(0.1, 0.2, 0.3, 0.4); }`;

    const s_vs = gl.createShader(gl.VERTEX_SHADER); gl.shaderSource(s_vs, vs); gl.compileShader(s_vs); assert.ok(gl.getShaderParameter(s_vs, gl.COMPILE_STATUS));
    const s_fs = gl.createShader(gl.FRAGMENT_SHADER); gl.shaderSource(s_fs, fs); gl.compileShader(s_fs); assert.ok(gl.getShaderParameter(s_fs, gl.COMPILE_STATUS));

    const prog = gl.createProgram(); gl.attachShader(prog, s_vs); gl.attachShader(prog, s_fs);
    gl.linkProgram(prog);
    assert.ok(gl.getProgramParameter(prog, gl.LINK_STATUS));

    // Use program and draw a point to trigger fragment
    gl.useProgram(prog);
    gl.clearColor(0,0,0,1);
    gl.clear(gl.COLOR_BUFFER_BIT);
    gl.drawArrays(gl.POINTS, 0, 1);

    // Read back the private memory at dynamic private_ptr
    const mem = gl._instance.exports.memory.buffer;
    const floatView = new Float32Array(mem, gl._scratchLayout.private, 4);

    // Due to float representation in memory the values should be near 0.1..0.4
    const got = Array.from(floatView).map(v => Math.round(v * 1000) / 1000);
    assert.deepStrictEqual(got, [0.1, 0.2, 0.3, 0.4]);

    // Also verify that readPixels returns the same color (rounding to bytes)
    const pixels = new Uint8Array(4); gl.readPixels(32,32,1,1,gl.RGBA, gl.UNSIGNED_BYTE, pixels);
    const expected = [Math.round(0.1*255), Math.round(0.2*255), Math.round(0.3*255), Math.round(0.4*255)];
    for (let i = 0; i < 4; i++) {
      assert.ok(Math.abs(pixels[i] - expected[i]) <= 1, `pixel channel ${i} out of tolerance: got ${pixels[i]} expected ${expected[i]}`);
    }

  } finally {
    gl.destroy();
  }
});