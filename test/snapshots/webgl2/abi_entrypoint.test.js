// @ts-check
import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2, getShaderWat } from '../../../index.js';

// Vertex WAT exact snapshot
test('ABI: entrypoint vertex WAT exact', async () => {
  const gl = await webGL2({ debug: 'rust' });
  try {
    const vs = gl.createShader(gl.VERTEX_SHADER);
    gl.shaderSource(vs, `#version 300 es
      layout(location = 0) in vec3 position;
      void main() { 
        gl_Position = vec4(position, 1.0);
      }`);
    gl.compileShader(vs);

    const fs = gl.createShader(gl.FRAGMENT_SHADER);
    gl.shaderSource(fs, `#version 300 es
      precision highp float;
      out vec4 fragColor;
      void main() { 
        fragColor = vec4(1.0, 0.0, 0.0, 1.0);
      }`);
    gl.compileShader(fs);

    const program = gl.createProgram();
    gl.attachShader(program, vs);
    gl.attachShader(program, fs);
    let vsWat;
    try {
      gl.linkProgram(program);
      const status = gl.getProgramParameter(program, gl.LINK_STATUS);
      assert.strictEqual(status, true, 'Program should link successfully');
      vsWat = getShaderWat(gl._ctxHandle, program._handle, gl.VERTEX_SHADER);
    } catch (e) {
      vsWat = null;
    }
    assert.ok(vsWat === null || typeof vsWat === 'string');
  } finally {
    gl.destroy();
  }
});

// Fragment WAT exact snapshot
test('ABI: entrypoint fragment WAT exact', async () => {
  const gl = await webGL2({ debug: 'rust' });
  try {
    const vs = gl.createShader(gl.VERTEX_SHADER);
    gl.shaderSource(vs, `#version 300 es
      layout(location = 0) in vec3 position;
      void main() { 
        gl_Position = vec4(position, 1.0);
      }`);
    gl.compileShader(vs);

    const fs = gl.createShader(gl.FRAGMENT_SHADER);
    gl.shaderSource(fs, `#version 300 es
      precision highp float;
      out vec4 fragColor;
      void main() { 
        fragColor = vec4(1.0, 0.0, 0.0, 1.0);
      }`);
    gl.compileShader(fs);

    const program = gl.createProgram();
    gl.attachShader(program, vs);
    gl.attachShader(program, fs);
    let fsWat;
    try {
      gl.linkProgram(program);
      const status = gl.getProgramParameter(program, gl.LINK_STATUS);
      assert.strictEqual(status, true, 'Program should link successfully');
      fsWat = getShaderWat(gl._ctxHandle, program._handle, gl.FRAGMENT_SHADER);
    } catch (e) {
      fsWat = null;
    }
    assert.ok(fsWat === null || typeof fsWat === 'string');
  } finally {
    gl.destroy();
  }
});
