import test from 'node:test';
import assert from 'node:assert';
import { webGL2 } from '../index.js';

// Focused per-component integer varying checks
test('Varying integer component checks', async (t) => {
  const gl = await webGL2();
  try {
    gl.viewport(0, 0, 64, 64);

    const vs = `#version 300 es
    layout(location=0) in ivec4 a;
    flat out ivec4 v0;
    void main(){ v0 = a; gl_Position = vec4(0.0); gl_PointSize = 1.0; }`;

    const fs = `#version 300 es
    precision highp float;
    flat in ivec4 v0; out vec4 fragColor;
    void main(){
      float r = (v0.x == -1) ? 1.0 : 0.0;
      float g = (v0.y == 2) ? 1.0 : 0.0;
      float b = (v0.z == -3) ? 1.0 : 0.0;
      fragColor = vec4(r,g,b,1.0);
    }`;

    const s_vs = gl.createShader(gl.VERTEX_SHADER); gl.shaderSource(s_vs, vs); gl.compileShader(s_vs); assert.ok(gl.getShaderParameter(s_vs, gl.COMPILE_STATUS));
    const s_fs = gl.createShader(gl.FRAGMENT_SHADER); gl.shaderSource(s_fs, fs); gl.compileShader(s_fs); assert.ok(gl.getShaderParameter(s_fs, gl.COMPILE_STATUS));
    const prog = gl.createProgram(); gl.attachShader(prog, s_vs); gl.attachShader(prog, s_fs); gl.linkProgram(prog);
    assert.ok(gl.getProgramParameter(prog, gl.LINK_STATUS)); gl.useProgram(prog);

    // Test components individually
    const cases = [
      { attr: [-1,0,0,0], expected: [255,0,0,255] },
      { attr: [0,2,0,0], expected: [0,255,0,255] },
      { attr: [0,0,-3,0], expected: [0,0,255,255] },
      { attr: [-1,2,-3,0], expected: [255,255,255,255] },
    ];

    for (const c of cases) {
      gl.vertexAttribI4i(0, c.attr[0], c.attr[1], c.attr[2], c.attr[3]);
      gl.clearColor(0,0,0,1); gl.clear(gl.COLOR_BUFFER_BIT);
      gl.drawArrays(gl.POINTS, 0, 1);
      const pixels = new Uint8Array(4); gl.readPixels(32,32,1,1,gl.RGBA, gl.UNSIGNED_BYTE, pixels);
      assert.deepStrictEqual(Array.from(pixels), c.expected);
    }

  } finally {
    gl.destroy();
  }
});