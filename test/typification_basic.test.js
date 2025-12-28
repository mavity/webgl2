import test from 'node:test';
import assert from 'node:assert';
import { webGL2 } from '../index.js';

test('Basic integer typification', async (t) => {
  const gl = await webGL2();
  gl.viewport(0, 0, 640, 480);

  await t.test('literal true in if', async () => {
    const vsSource = `#version 300 es
      void main() {
        gl_Position = vec4(0.0, 0.0, 0.0, 1.0);
      }`;

    const fsSource = `#version 300 es
      precision highp float;
      out vec4 fragColor;
      void main() {
        if (true) {
          fragColor = vec4(0.0, 1.0, 0.0, 1.0); // Green
        } else {
          fragColor = vec4(1.0, 0.0, 0.0, 1.0); // Red
        }
      }`;

    const program = gl.createProgram();
    const vs = gl.createShader(gl.VERTEX_SHADER);
    gl.shaderSource(vs, vsSource);
    gl.compileShader(vs);
    gl.attachShader(program, vs);

    const fs = gl.createShader(gl.FRAGMENT_SHADER);
    gl.shaderSource(fs, fsSource);
    gl.compileShader(fs);
    if (!gl.getShaderParameter(fs, gl.COMPILE_STATUS)) {
      throw new Error('FS compile failed: ' + gl.getShaderInfoLog(fs));
    }
    gl.attachShader(program, fs);

    gl.linkProgram(program);
    if (!gl.getProgramParameter(program, gl.LINK_STATUS)) {
      throw new Error('Link failed: ' + gl.getProgramInfoLog(program));
    }
    gl.useProgram(program);

    gl.clearColor(0, 0, 0, 1);
    gl.clear(gl.COLOR_BUFFER_BIT);
    gl.drawArrays(gl.POINTS, 0, 1);

    const pixels = new Uint8Array(4);
    gl.readPixels(320, 240, 1, 1, gl.RGBA, gl.UNSIGNED_BYTE, pixels);
    console.log('Literal true test pixels:', Array.from(pixels));
    assert.deepStrictEqual(Array.from(pixels), [0, 255, 0, 255], 'Should be green');
  });

  await t.test('integer vertex attribute', async () => {
    const vsSource = `#version 300 es
      layout(location = 0) in int a_val;
      flat out int v_val;
      void main() {
        v_val = a_val;
        gl_Position = vec4(0.0, 0.0, 0.0, 1.0);
      }`;

    const fsSource = `#version 300 es
      precision highp float;
      flat in int v_val;
      out vec4 fragColor;
      void main() {
        if (v_val == 42) {
          fragColor = vec4(0.0, 1.0, 0.0, 1.0); // Green
        } else {
          fragColor = vec4(1.0, 0.0, 0.0, 1.0); // Red
        }
      }`;

    const program = gl.createProgram();
    const vs = gl.createShader(gl.VERTEX_SHADER);
    gl.shaderSource(vs, vsSource);
    gl.compileShader(vs);
    if (!gl.getShaderParameter(vs, gl.COMPILE_STATUS)) {
      throw new Error('VS compile failed: ' + gl.getShaderInfoLog(vs));
    }
    gl.attachShader(program, vs);

    const fs = gl.createShader(gl.FRAGMENT_SHADER);
    gl.shaderSource(fs, fsSource);
    gl.compileShader(fs);
    if (!gl.getShaderParameter(fs, gl.COMPILE_STATUS)) {
      throw new Error('FS compile failed: ' + gl.getShaderInfoLog(fs));
    }
    gl.attachShader(program, fs);

    gl.linkProgram(program);
    if (!gl.getProgramParameter(program, gl.LINK_STATUS)) {
      throw new Error('Link failed: ' + gl.getProgramInfoLog(program));
    }
    gl.useProgram(program);

    // Set integer attribute via vertexAttribI
    gl.vertexAttribI4i(0, 42, 0, 0, 0);

    gl.clearColor(0, 0, 0, 1);
    gl.clear(gl.COLOR_BUFFER_BIT);
    gl.drawArrays(gl.POINTS, 0, 1);

    const pixels = new Uint8Array(4);
    gl.readPixels(320, 240, 1, 1, gl.RGBA, gl.UNSIGNED_BYTE, pixels);
    console.log('Integer vertex attribute test pixels:', Array.from(pixels));
    assert.deepStrictEqual(Array.from(pixels), [0, 255, 0, 255], 'Should be green');
  });
});
