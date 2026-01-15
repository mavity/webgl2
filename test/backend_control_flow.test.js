import test from 'node:test';
import assert from 'node:assert';
import { webGL2 } from '../index.js';

test('Backend Control Flow', async (t) => {
  const gl = await webGL2();
  
  t.after(() => {
    gl.destroy();
  });
  
  function createProgram(vsSrc, fsSrc) {
      const vs = gl.createShader(gl.VERTEX_SHADER);
      gl.shaderSource(vs, vsSrc);
      gl.compileShader(vs);
      if (!gl.getShaderParameter(vs, gl.COMPILE_STATUS)) throw new Error('VS: ' + gl.getShaderInfoLog(vs));
      const fs = gl.createShader(gl.FRAGMENT_SHADER);
      gl.shaderSource(fs, fsSrc);
      gl.compileShader(fs);
      if (!gl.getShaderParameter(fs, gl.COMPILE_STATUS)) throw new Error('FS: ' + gl.getShaderInfoLog(fs));
      const p = gl.createProgram();
      gl.attachShader(p, vs);
      gl.attachShader(p, fs);
      gl.linkProgram(p);
      if (!gl.getProgramParameter(p, gl.LINK_STATUS)) throw new Error('Link: ' + gl.getProgramInfoLog(p));
      return p;
  }

  const vsCheck = `#version 300 es
  void main() {
      gl_Position = vec4(0.0, 0.0, 0.0, 1.0);
      gl_PointSize = 100.0;
  }`;

  async function runTest(fsSrc) {
      const p = createProgram(vsCheck, fsSrc);
      gl.useProgram(p);
      gl.clearColor(0.2, 0.4, 0.6, 1.0);
      gl.clear(gl.COLOR_BUFFER_BIT);
      gl.drawArrays(gl.POINTS, 0, 1);
      const pixel = new Uint8Array(4);
      gl.readPixels(320, 240, 1, 1, gl.RGBA, gl.UNSIGNED_BYTE, pixel);
      return Array.from(pixel);
  }

  await t.test('Simple IF', async () => {
      const fs = `#version 300 es
      precision mediump float;
      out vec4 color;
      void main() {
          int x = 1;
          int acc = 5;
          if (x == 1) acc = 10;
          if (acc == 10) color = vec4(0.0, 1.0, 0.0, 1.0);
          else color = vec4(1.0, 0.0, 0.0, 1.0);
      }`;
      const res = await runTest(fs);
      assert.deepStrictEqual(res, [0, 255, 0, 255]);
  });
});
