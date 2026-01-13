// @ts-check
import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2, getShaderWat } from '../../../index.js';

test('ABI: small constant array should not produce WAT (backend unsupported)', async () => {
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
    const fsWat = getShaderWat(gl._ctxHandle, program._handle, gl.FRAGMENT_SHADER);
    // not supported??
    assert.strictEqual(fsWat, null);
  } finally {
    gl.destroy();
  }
});

test('ABI: large constant array should not produce WAT (backend unsupported)', async () => {
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
    const fsWat = getShaderWat(gl._ctxHandle, program._handle, gl.FRAGMENT_SHADER);
    // not supported!
    assert.strictEqual(fsWat, null);
  } finally {
    gl.destroy();
  }
});

test('ABI: vec4 array should not produce WAT (backend unsupported)', async () => {
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

    const fsWat = getShaderWat(gl._ctxHandle, program._handle, gl.FRAGMENT_SHADER);
    assert.strictEqual(fsWat, null);
  } finally {
    gl.destroy();
  }
});