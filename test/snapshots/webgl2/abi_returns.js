// @ts-check
// ABI Test: Return value handling
// Tests scalar returns, vector returns, and large struct returns

import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2, getShaderWat } from '../../../index.js';

test('ABI: vec4 return (flattened)', async () => {
  const gl = await webGL2({ debug: 'rust' });

  try {
    const fs = gl.createShader(gl.FRAGMENT_SHADER);
    gl.shaderSource(fs, `#version 300 es
      precision highp float;
      out vec4 fragColor;
      
      vec4 makeColor(float r, float g, float b) {
        return vec4(r, g, b, 1.0);
      }
      
      void main() {
        fragColor = makeColor(1.0, 0.5, 0.0);
      }`);
    gl.compileShader(fs);

    const vs = gl.createShader(gl.VERTEX_SHADER);
    gl.shaderSource(vs, `#version 300 es
      void main() { gl_Position = vec4(0); }`);
    gl.compileShader(vs);

    const program = gl.createProgram();
    gl.attachShader(program, vs);
    gl.attachShader(program, fs);
    gl.linkProgram(program);

    const status = gl.getProgramParameter(program, gl.LINK_STATUS);
    assert.strictEqual(status, true, 'Program with vec4 return should link successfully');

    const fsWat = getShaderWat(gl._ctxHandle, program._handle, gl.FRAGMENT_SHADER);
    
    // vec4 (16 bytes) should be flattened to 4 f32 returns
    assert.match(fsWat, /\(type.*\(func \(param f32 f32 f32\) \(result f32 f32 f32 f32\)/, 
      'makeColor should return 4 f32 values');
    
  } finally {
    gl.destroy();
  }
});

test('ABI: mat3 return', async () => {
  const gl = await webGL2({ debug: 'rust' });

  try {
    const fs = gl.createShader(gl.FRAGMENT_SHADER);
    gl.shaderSource(fs, `#version 300 es
      precision highp float;
      out vec4 fragColor;
      
      mat3 makeMatrix() {
        return mat3(
          1.0, 2.0, 3.0,
          4.0, 5.0, 6.0,
          7.0, 8.0, 9.0
        );
      }
      
      void main() {
        mat3 m = makeMatrix();
        fragColor = vec4(m[0][0], m[1][1], m[2][2], 1.0);
      }`);
    gl.compileShader(fs);

    const vs = gl.createShader(gl.VERTEX_SHADER);
    gl.shaderSource(vs, `#version 300 es
      void main() { gl_Position = vec4(0); }`);
    gl.compileShader(vs);

    const program = gl.createProgram();
    gl.attachShader(program, vs);
    gl.attachShader(program, fs);
    gl.linkProgram(program);

    const status = gl.getProgramParameter(program, gl.LINK_STATUS);
    assert.strictEqual(status, true, 'Program with mat3 return should link successfully');

    const fsWat = getShaderWat(gl._ctxHandle, program._handle, gl.FRAGMENT_SHADER);
    
    // mat3 (36 bytes) might use frame for return
    console.log('mat3 return WAT preview (first 1500 chars):');
    console.log(fsWat.substring(0, 1500));
    
  } finally {
    gl.destroy();
  }
});

test('ABI: large struct return', async () => {
  const gl = await webGL2({ debug: 'rust' });

  try {
    const fs = gl.createShader(gl.FRAGMENT_SHADER);
    gl.shaderSource(fs, `#version 300 es
      precision highp float;
      out vec4 fragColor;
      
      struct LargeResult {
        vec4 color1;
        vec4 color2;
        vec4 color3;
      };
      
      LargeResult makeResult() {
        LargeResult res;
        res.color1 = vec4(1.0, 0.0, 0.0, 1.0);
        res.color2 = vec4(0.0, 1.0, 0.0, 1.0);
        res.color3 = vec4(0.0, 0.0, 1.0, 1.0);
        return res;
      }
      
      void main() {
        LargeResult result = makeResult();
        fragColor = result.color1 + result.color2 + result.color3;
      }`);
    gl.compileShader(fs);

    const vs = gl.createShader(gl.VERTEX_SHADER);
    gl.shaderSource(vs, `#version 300 es
      void main() { gl_Position = vec4(0); }`);
    gl.compileShader(vs);

    const program = gl.createProgram();
    gl.attachShader(program, vs);
    gl.attachShader(program, fs);
    gl.linkProgram(program);

    const status = gl.getProgramParameter(program, gl.LINK_STATUS);
    assert.strictEqual(status, true, 'Program with large struct return should link successfully');

    const fsWat = getShaderWat(gl._ctxHandle, program._handle, gl.FRAGMENT_SHADER);
    
    // Large struct (48 bytes) return might use frame or be flattened
    console.log('Large struct return WAT preview (first 2000 chars):');
    console.log(fsWat.substring(0, 2000));
    
  } finally {
    gl.destroy();
  }
});
