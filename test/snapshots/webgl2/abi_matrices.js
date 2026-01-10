// @ts-check
// ABI Test: Matrix parameters and returns
// Tests various matrix sizes and their classification

import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2, getShaderWat } from '../../../index.js';

test('ABI: mat2 parameter (flattened)', async () => {
  const gl = await webGL2({ debug: 'rust' });

  try {
    const fs = gl.createShader(gl.FRAGMENT_SHADER);
    gl.shaderSource(fs, `#version 300 es
      precision highp float;
      out vec4 fragColor;
      
      float processMat2(mat2 m) {
        return m[0][0] + m[1][1];
      }
      
      void main() {
        mat2 m = mat2(1.0, 2.0, 3.0, 4.0);
        float result = processMat2(m);
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
    assert.strictEqual(status, true, 'Program with mat2 should link successfully');

    const fsWat = getShaderWat(gl._ctxHandle, program._handle, gl.FRAGMENT_SHADER);
    
    // mat2 (16 bytes) should be at threshold, likely flattened
    assert.match(fsWat, /\(type.*\(func \(param f32 f32 f32 f32\)/, 
      'processMat2 should take 4 f32 parameters (flattened)');
    
  } finally {
    gl.destroy();
  }
});

test('ABI: mat4 parameter (frame)', async () => {
  const gl = await webGL2({ debug: 'rust' });

  try {
    const fs = gl.createShader(gl.FRAGMENT_SHADER);
    gl.shaderSource(fs, `#version 300 es
      precision highp float;
      out vec4 fragColor;
      
      vec4 transformVector(mat4 m, vec4 v) {
        return m * v;
      }
      
      void main() {
        mat4 m = mat4(1.0);
        vec4 v = vec4(1.0, 2.0, 3.0, 4.0);
        fragColor = transformVector(m, v);
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
    assert.strictEqual(status, true, 'Program with mat4 should link successfully');

    const fsWat = getShaderWat(gl._ctxHandle, program._handle, gl.FRAGMENT_SHADER);
    
    // mat4 (64 bytes) should definitely use frame
    assert.match(fsWat, /global\.get.*5/, 'Should use frame for mat4');
    assert.match(fsWat, /\(type.*\(func \(param i32 f32 f32 f32 f32\)/, 
      'transformVector should take i32 pointer for mat4 and 4 f32 for vec4');
    
  } finally {
    gl.destroy();
  }
});

test('ABI: mat3 parameter', async () => {
  const gl = await webGL2({ debug: 'rust' });

  try {
    const fs = gl.createShader(gl.FRAGMENT_SHADER);
    gl.shaderSource(fs, `#version 300 es
      precision highp float;
      out vec4 fragColor;
      
      vec3 transformVec3(mat3 m, vec3 v) {
        return m * v;
      }
      
      void main() {
        mat3 m = mat3(1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0);
        vec3 v = vec3(1.0, 2.0, 3.0);
        vec3 result = transformVec3(m, v);
        fragColor = vec4(result, 1.0);
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
    assert.strictEqual(status, true, 'Program with mat3 should link successfully');

    const fsWat = getShaderWat(gl._ctxHandle, program._handle, gl.FRAGMENT_SHADER);
    
    // mat3 (36 bytes) should use frame
    assert.match(fsWat, /global\.get.*5/, 'Should use frame for mat3');
    
    console.log('mat3 parameter WAT preview (first 1500 chars):');
    console.log(fsWat.substring(0, 1500));
    
  } finally {
    gl.destroy();
  }
});

test('ABI: multiple matrix parameters', async () => {
  const gl = await webGL2({ debug: 'rust' });

  try {
    const fs = gl.createShader(gl.FRAGMENT_SHADER);
    gl.shaderSource(fs, `#version 300 es
      precision highp float;
      out vec4 fragColor;
      
      mat4 multiplyMatrices(mat4 a, mat4 b) {
        return a * b;
      }
      
      void main() {
        mat4 m1 = mat4(1.0);
        mat4 m2 = mat4(2.0);
        mat4 result = multiplyMatrices(m1, m2);
        fragColor = vec4(result[0][0], result[1][1], result[2][2], result[3][3]);
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
    assert.strictEqual(status, true, 'Program with multiple mat4 should link successfully');

    const fsWat = getShaderWat(gl._ctxHandle, program._handle, gl.FRAGMENT_SHADER);
    
    // Two mat4 params should both use frame (two i32 pointers)
    assert.match(fsWat, /\(type.*\(func \(param i32 i32\)/, 
      'multiplyMatrices should take two i32 pointers');
    
  } finally {
    gl.destroy();
  }
});
