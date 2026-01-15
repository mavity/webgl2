import test from 'node:test';
import assert from 'node:assert';
import { webGL2 } from '../index.js';

test('Backend Control Flow - Group 2.2', async (t) => {
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

  // Group 2.1: Block + Break (already validated)
  await t.test('Block with Break', async () => {
      const fs = `#version 300 es
      precision mediump float;
      out vec4 color;
      void main() {
          int x = 0;
          for(int i=0; i<10; i++) {
              if (i == 5) break;
              x++;
          }
          if (x == 5) color = vec4(0.0, 1.0, 0.0, 1.0);
          else color = vec4(1.0, 0.0, 0.0, 1.0);
      }`;
      const res = await runTest(fs);
      assert.deepStrictEqual(res, [0, 255, 0, 255]);
  });

  // Group 2.2: Statement::Loop
  await t.test('Loop - Basic Iteration', async () => {
      const fs = `#version 300 es
      precision mediump float;
      out vec4 color;
      void main() {
          int sum = 0;
          for(int i=0; i<5; i++) {
              sum += i;
          }
          if (sum == 10) color = vec4(0.0, 1.0, 0.0, 1.0);
          else color = vec4(1.0, 0.0, 0.0, 1.0);
      }`;
      const res = await runTest(fs);
      assert.deepStrictEqual(res, [0, 255, 0, 255]);
  });

  // Group 2.2: Statement::Continue
  await t.test('Loop with Continue', async () => {
      const fs = `#version 300 es
      precision mediump float;
      out vec4 color;
      void main() {
          int x = 0;
          for(int i=0; i<10; i++) {
              if (i == 0 || i == 2 || i == 4 || i == 6 || i == 8) continue;
              x += 1;
          }
          if (x == 5) color = vec4(0.0, 1.0, 0.0, 1.0);
          else color = vec4(1.0, 0.0, 0.0, 1.0);
      }`;
      const res = await runTest(fs);
      assert.deepStrictEqual(res, [0, 255, 0, 255]);
  });

  // Group 2.2: Statement::Switch - Multiple Cases
  await t.test('Switch with Multiple Cases', async () => {
      const fs = `#version 300 es
      precision mediump float;
      out vec4 color;
      void main() {
          int x = 2;
          int acc = 0;
          switch(x) {
              case 1: acc = 10; break;
              case 2: acc = 20; break;
              case 3: acc = 30; break;
          }
          if (acc == 20) color = vec4(0.0, 1.0, 0.0, 1.0);
          else color = vec4(1.0, 0.0, 0.0, 1.0);
      }`;
      const res = await runTest(fs);
      assert.deepStrictEqual(res, [0, 255, 0, 255]);
  });

  // Group 2.2: Statement::Switch - Default Case
  await t.test('Switch with Default', async () => {
      const fs = `#version 300 es
      precision mediump float;
      out vec4 color;
      void main() {
          int x = 5;
          int acc = 0;
          switch(x) {
              case 1: acc = 10; break;
              default: acc = 50; break;
          }
          if (acc == 50) color = vec4(0.0, 1.0, 0.0, 1.0);
          else color = vec4(1.0, 0.0, 0.0, 1.0);
      }`;
      const res = await runTest(fs);
      assert.deepStrictEqual(res, [0, 255, 0, 255]);
  });

  // Group 2.2: Statement::Switch - Fallthrough
  await t.test('Switch with Fallthrough', async () => {
      const fs = `#version 300 es
      precision mediump float;
      out vec4 color;
      void main() {
          int x = 1;
          int acc = 0;
          switch(x) {
              case 1: // fallthrough
              case 2: acc = 20; break;
              case 3: acc = 30; break;
          }
          if (acc == 20) color = vec4(0.0, 1.0, 0.0, 1.0);
          else color = vec4(1.0, 0.0, 0.0, 1.0);
      }`;
      const res = await runTest(fs);
      assert.deepStrictEqual(res, [0, 255, 0, 255]);
  });

  // Group 2.2: Statement::Kill
  await t.test('Kill Statement (discard)', async () => {
      const fs = `#version 300 es
      precision mediump float;
      out vec4 color;
      void main() {
          if (gl_FragCoord.x < 0.0) {
              discard;
          }
          color = vec4(0.0, 1.0, 0.0, 1.0);
      }`;
      const res = await runTest(fs);
      // Since gl_FragCoord.x at center (320) is positive, discard won't execute
      assert.deepStrictEqual(res, [0, 255, 0, 255]);
  });

  // Comprehensive: Nested Control Flow
  await t.test('Nested Loops and Switch', async () => {
      const fs = `#version 300 es
      precision mediump float;
      out vec4 color;
      void main() {
          int acc = 0;
          for (int i = 0; i < 2; i++) {
              for (int j = 0; j < 3; j++) {
                  switch(i + j) {
                      case 1: acc += 10; break;
                      case 2: acc += 100; break;
                      default: acc += 1; break;
                  }
              }
          }
          if (acc == 222) color = vec4(0.0, 1.0, 0.0, 1.0);
          else color = vec4(1.0, 0.0, 0.0, 1.0);
      }`;
      const res = await runTest(fs);
      assert.deepStrictEqual(res, [0, 255, 0, 255]);
  });
});
