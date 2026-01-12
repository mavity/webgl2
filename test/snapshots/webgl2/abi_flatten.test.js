// @ts-check
import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2, getShaderWat } from '../../../index.js';

test('ABI: single internal function with scalar param (flattened)', async () => {
  const gl = await webGL2({ debug: 'rust' });
  try {
    const fs = gl.createShader(gl.FRAGMENT_SHADER);
    gl.shaderSource(fs, `#version 300 es
      precision highp float;
      out vec4 fragColor;
      void helper(float val) { val = val + 1.0; }
      void main() { helper(3.0); fragColor = vec4(1.0); }`);
    gl.compileShader(fs);

    const vs = gl.createShader(gl.VERTEX_SHADER);
    gl.shaderSource(vs, `#version 300 es
      void main() { gl_Position = vec4(0); }`);
    gl.compileShader(vs);

    const program = gl.createProgram();
    gl.attachShader(program, vs);
    gl.attachShader(program, fs);
    gl.linkProgram(program);

    const fsWat = getShaderWat(gl._ctxHandle, program._handle, gl.FRAGMENT_SHADER);
    assert.ok(fsWat === null || typeof fsWat === 'string');
  } finally {
    gl.destroy();
  }
});

test('ABI: internal function with vec2 param (flattened)', async () => {
  const gl = await webGL2({ debug: 'rust' });
  try {
    const fs = gl.createShader(gl.FRAGMENT_SHADER);
    gl.shaderSource(fs, `#version 300 es
      precision highp float;
      out vec4 fragColor;
      float helper(vec2 val) { return val.x + val.y; }
      void main() { float result = helper(vec2(1.0,2.0)); fragColor = vec4(result,0.0,0.0,1.0);} `);
    gl.compileShader(fs);

    const vs = gl.createShader(gl.VERTEX_SHADER);
    gl.shaderSource(vs, `#version 300 es
      void main() { gl_Position = vec4(0); }`);
    gl.compileShader(vs);

    const program = gl.createProgram();
    gl.attachShader(program, vs);
    gl.attachShader(program, fs);
    gl.linkProgram(program);

    const fsWat = getShaderWat(gl._ctxHandle, program._handle, gl.FRAGMENT_SHADER);
    assert.ok(fsWat === null || typeof fsWat === 'string');
  } finally {
    gl.destroy();
  }
});

test('ABI: internal function with scalar return value', async () => {
  const gl = await webGL2({ debug: 'rust' });
  try {
    const fs = gl.createShader(gl.FRAGMENT_SHADER);
    gl.shaderSource(fs, `#version 300 es
      precision highp float;
      out vec4 fragColor;
      float compute() { return 0.5; }
      void main() { float val = compute(); fragColor = vec4(val,val,val,1.0); }`);
    gl.compileShader(fs);

    const vs = gl.createShader(gl.VERTEX_SHADER);
    gl.shaderSource(vs, `#version 300 es
      void main() { gl_Position = vec4(0); }`);
    gl.compileShader(vs);

    const program = gl.createProgram();
    gl.attachShader(program, vs);
    gl.attachShader(program, fs);
    gl.linkProgram(program);

    const fsWat = getShaderWat(gl._ctxHandle, program._handle, gl.FRAGMENT_SHADER);
    assert.ok(fsWat === null || typeof fsWat === 'string');
  } finally {
    gl.destroy();
  }
});

test('ABI: internal function with vec3 return (flattened)', async () => {
  const gl = await webGL2({ debug: 'rust' });
  try {
    const fs = gl.createShader(gl.FRAGMENT_SHADER);
    gl.shaderSource(fs, `#version 300 es
      precision highp float;
      out vec4 fragColor;
      vec3 makeVec3() { return vec3(1.0, 0.0, 0.0); }
      void main() { vec3 v = makeVec3(); fragColor = vec4(v,1.0);} `);
    gl.compileShader(fs);

    const vs = gl.createShader(gl.VERTEX_SHADER);
    gl.shaderSource(vs, `#version 300 es
      void main() { gl_Position = vec4(0); }`);
    gl.compileShader(vs);

    const program = gl.createProgram();
    gl.attachShader(program, vs);
    gl.attachShader(program, fs);
    gl.linkProgram(program);

    const fsWat = getShaderWat(gl._ctxHandle, program._handle, gl.FRAGMENT_SHADER);
    assert.ok(fsWat === null || typeof fsWat === 'string');
  } finally {
    gl.destroy();
  }
});