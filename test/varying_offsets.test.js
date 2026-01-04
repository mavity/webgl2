import test from 'node:test';
import assert from 'node:assert';
import { webGL2 } from '../index.js';

// Check varying layout across locations
test('Varying location offset checks', async (t) => {
  const gl = await webGL2();
  try {
    gl.viewport(0, 0, 64, 64);

    const vs = `#version 300 es
    layout(location=0) in ivec4 a;
    layout(location=1) in ivec4 b;
    flat out ivec4 va; flat out ivec4 vb;
    void main(){ va = a; vb = b; gl_Position = vec4(0.0, 0.0, 0.0, 1.0); gl_PointSize = 1.0; }`;

    const fs = `#version 300 es
    precision highp float;
    flat in ivec4 va; flat in ivec4 vb; out vec4 fragColor;
    void main(){
      if (va.x == -1 && vb.x == 1) {
          fragColor = vec4(0,1,0,1);
      } else {
          fragColor = vec4(1,0,0,1);
      }
    }`;

    const s_vs = gl.createShader(gl.VERTEX_SHADER); gl.shaderSource(s_vs, vs); gl.compileShader(s_vs); assert.ok(gl.getShaderParameter(s_vs, gl.COMPILE_STATUS));
    const s_fs = gl.createShader(gl.FRAGMENT_SHADER); gl.shaderSource(s_fs, fs); gl.compileShader(s_fs); assert.ok(gl.getShaderParameter(s_fs, gl.COMPILE_STATUS));
    const prog = gl.createProgram(); gl.attachShader(prog, s_vs); gl.attachShader(prog, s_fs); gl.linkProgram(prog);
    
    if (!gl.getProgramParameter(prog, gl.LINK_STATUS)) {
        console.log('Link failed:', gl.getProgramInfoLog(prog));
    }
    assert.ok(gl.getProgramParameter(prog, gl.LINK_STATUS)); gl.useProgram(prog);

    // Set a and b attributes
    gl.vertexAttribI4i(0, -1,0,0,0);
    gl.vertexAttribI4i(1, 1,0,0,0);

    gl.clear(gl.COLOR_BUFFER_BIT);
    // Use nested ifs in shader to avoid unsupported LogicalAnd operator
    gl.drawArrays(gl.POINTS, 0, 1);
    const pixels = new Uint8Array(4); gl.readPixels(32,32,1,1,gl.RGBA, gl.UNSIGNED_BYTE, pixels);
    assert.deepStrictEqual(Array.from(pixels), [0,255,0,255]);

  } finally {
    gl.destroy();
  }
});