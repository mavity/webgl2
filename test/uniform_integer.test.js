import test from 'node:test';
import assert from 'node:assert';
import { webGL2 } from '../index.js';

// Verify uniform integer comparison works (isolate varying transport)
test('Uniform integer compare check', { skip: true }, async (t) => {
  const gl = await webGL2();
  try {
    gl.viewport(0, 0, 64, 64);

    const vs = `#version 300 es
    void main(){ gl_Position = vec4(0.0); gl_PointSize = 1.0; }`;

    const fs = `#version 300 es
    precision highp float;
    uniform int u0; uniform int u1; uniform int u2; uniform int u3; out vec4 fragColor;
    void main(){
      fragColor = vec4(1,0,0,1);
      if (u0 == -1) {
        if (u1 == 2) {
          if (u2 == -3) {
            if (u3 == 4) {
              fragColor = vec4(0,1,0,1);
            }
          }
        }
      }
    }`;

    const s_vs = gl.createShader(gl.VERTEX_SHADER); gl.shaderSource(s_vs, vs); gl.compileShader(s_vs); assert.ok(gl.getShaderParameter(s_vs, gl.COMPILE_STATUS));
    const s_fs = gl.createShader(gl.FRAGMENT_SHADER); gl.shaderSource(s_fs, fs); gl.compileShader(s_fs); assert.ok(gl.getShaderParameter(s_fs, gl.COMPILE_STATUS));
    const prog = gl.createProgram(); gl.attachShader(prog, s_vs); gl.attachShader(prog, s_fs); gl.linkProgram(prog);
    assert.ok(gl.getProgramParameter(prog, gl.LINK_STATUS)); gl.useProgram(prog);

    const loc0 = gl.getUniformLocation(prog, 'u0');
    const loc1 = gl.getUniformLocation(prog, 'u1');
    const loc2 = gl.getUniformLocation(prog, 'u2');
    const loc3 = gl.getUniformLocation(prog, 'u3');
    gl.uniform1i(loc0, -1); gl.uniform1i(loc1, 2); gl.uniform1i(loc2, -3); gl.uniform1i(loc3, 4);

    gl.clearColor(0, 0, 0, 1); gl.clear(gl.COLOR_BUFFER_BIT);
    gl.drawArrays(gl.POINTS, 0, 1);
    const pixels = new Uint8Array(4); gl.readPixels(32, 32, 1, 1, gl.RGBA, gl.UNSIGNED_BYTE, pixels);
    assert.deepStrictEqual(Array.from(pixels), [0, 255, 0, 255]);

  } finally {
    gl.destroy();
  }
});