// @ts-check
// ABI Test: Array parameters and return values
// Tests constant arrays, small vs large arrays, and array flattening

import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2, getShaderWat } from '../../../index.js';

test('ABI: small constant array (flattened)', async () => {
  const gl = await webGL2({ debug: 'rust' });

  try {
    const fs = gl.createShader(gl.FRAGMENT_SHADER);
    gl.shaderSource(fs, `#version 300 es
      precision highp float;
      out vec4 fragColor;
      
      float sumArray(float arr[4]) {
        return arr[0] + arr[1] + arr[2] + arr[3];
      }
      
      void main() {
        float data[4];
        data[0] = 1.0;
        data[1] = 2.0;
        data[2] = 3.0;
        data[3] = 4.0;
        
        float result = sumArray(data);
        fragColor = vec4(result, 0.0, 0.0, 1.0);
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
    assert.strictEqual(status, true, 'Program with small array should link successfully');

    const fsWat = getShaderWat(gl._ctxHandle, program._handle, gl.FRAGMENT_SHADER);
    
    // Small array (4 floats = 16 bytes) should be at the threshold
    console.log('Small array WAT preview (first 1500 chars):');
    console.log(fsWat.substring(0, 1500));
    
  } finally {
    gl.destroy();
  }
});

test('ABI: large constant array (frame)', async () => {
  const gl = await webGL2({ debug: 'rust' });

  try {
    const fs = gl.createShader(gl.FRAGMENT_SHADER);
    gl.shaderSource(fs, `#version 300 es
      precision highp float;
      out vec4 fragColor;
      
      float sumLargeArray(float arr[10]) {
        float sum = 0.0;
        for (int i = 0; i < 10; i++) {
          sum += arr[i];
        }
        return sum;
      }
      
      void main() {
        float data[10];
        for (int i = 0; i < 10; i++) {
          data[i] = float(i);
        }
        
        float result = sumLargeArray(data);
        fragColor = vec4(result, 0.0, 0.0, 1.0);
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
    assert.strictEqual(status, true, 'Program with large array should link successfully');

    const fsWat = getShaderWat(gl._ctxHandle, program._handle, gl.FRAGMENT_SHADER);
    
    // Large array (10 floats = 40 bytes) should use frame
    assert.match(fsWat, /global\.get.*5/, 'Should use frame stack for large array');
    assert.match(fsWat, /\(type.*\(func \(param i32\)/, 'sumLargeArray should take i32 pointer');
    
  } finally {
    gl.destroy();
  }
});

test('ABI: vec4 array', async () => {
  const gl = await webGL2({ debug: 'rust' });

  try {
    const fs = gl.createShader(gl.FRAGMENT_SHADER);
    gl.shaderSource(fs, `#version 300 es
      precision highp float;
      out vec4 fragColor;
      
      vec4 processVecArray(vec4 arr[3]) {
        return arr[0] + arr[1] + arr[2];
      }
      
      void main() {
        vec4 data[3];
        data[0] = vec4(1.0, 2.0, 3.0, 4.0);
        data[1] = vec4(5.0, 6.0, 7.0, 8.0);
        data[2] = vec4(9.0, 10.0, 11.0, 12.0);
        
        fragColor = processVecArray(data);
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
    assert.strictEqual(status, true, 'Program with vec4 array should link successfully');

    const fsWat = getShaderWat(gl._ctxHandle, program._handle, gl.FRAGMENT_SHADER);
    
    // vec4[3] = 48 bytes, should use frame
    assert.match(fsWat, /global\.get.*5/, 'Should use frame stack for vec4 array');
    
  } finally {
    gl.destroy();
  }
});
