// @ts-check
// ABI Test: Mixed parameter types (flattened + frame)
// Tests functions with both small and large parameters

import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2, getShaderWat } from '../../../index.js';

test('ABI: mixed small and large parameters', async () => {
  const gl = await webGL2({ debug: 'rust' });

  try {
    const fs = gl.createShader(gl.FRAGMENT_SHADER);
    gl.shaderSource(fs, `#version 300 es
      precision highp float;
      out vec4 fragColor;
      
      struct LargeData {
        vec4 a;
        vec4 b;
        vec4 c;
      };
      
      float mixedParams(float scalar, vec2 small, LargeData large, int flag) {
        if (flag > 0) {
          return scalar + small.x + large.a.x + large.b.y + large.c.z;
        }
        return 0.0;
      }
      
      void main() {
        LargeData data;
        data.a = vec4(1.0, 2.0, 3.0, 4.0);
        data.b = vec4(5.0, 6.0, 7.0, 8.0);
        data.c = vec4(9.0, 10.0, 11.0, 12.0);
        
        float result = mixedParams(10.0, vec2(20.0, 30.0), data, 1);
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
    assert.strictEqual(status, true, 'Program with mixed params should link successfully');

    const fsWat = getShaderWat(gl._ctxHandle, program._handle, gl.FRAGMENT_SHADER);
    
    // Should have both flattened params (scalar, vec2, flag) and frame param (large struct)
    assert.match(fsWat, /\(type.*\(func \(param f32 f32 f32 i32 i32\)/, 
      'mixedParams should have flattened scalars and i32 pointer for large struct');
    assert.match(fsWat, /global\.get.*5/, 'Should use frame allocation for large param');
    
    console.log('Mixed params WAT preview (first 2000 chars):');
    console.log(fsWat.substring(0, 2000));
    
  } finally {
    gl.destroy();
  }
});

test('ABI: multiple large parameters', async () => {
  const gl = await webGL2({ debug: 'rust' });

  try {
    const fs = gl.createShader(gl.FRAGMENT_SHADER);
    gl.shaderSource(fs, `#version 300 es
      precision highp float;
      out vec4 fragColor;
      
      struct Data1 {
        vec4 a;
        vec4 b;
        vec4 c;
      };
      
      struct Data2 {
        vec4 x;
        vec4 y;
        vec4 z;
      };
      
      float combineLarge(Data1 d1, Data2 d2) {
        return d1.a.x + d1.b.y + d1.c.z + d2.x.w + d2.y.x + d2.z.y;
      }
      
      void main() {
        Data1 data1;
        data1.a = vec4(1.0, 2.0, 3.0, 4.0);
        data1.b = vec4(5.0, 6.0, 7.0, 8.0);
        data1.c = vec4(9.0, 10.0, 11.0, 12.0);
        
        Data2 data2;
        data2.x = vec4(13.0, 14.0, 15.0, 16.0);
        data2.y = vec4(17.0, 18.0, 19.0, 20.0);
        data2.z = vec4(21.0, 22.0, 23.0, 24.0);
        
        float result = combineLarge(data1, data2);
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
    assert.strictEqual(status, true, 'Program with multiple large params should link successfully');

    const fsWat = getShaderWat(gl._ctxHandle, program._handle, gl.FRAGMENT_SHADER);
    
    // Should have two i32 pointer parameters
    assert.match(fsWat, /\(type.*\(func \(param i32 i32\)/, 
      'combineLarge should take two i32 pointers');
    assert.match(fsWat, /global\.get.*5/, 'Should use frame allocation');
    
  } finally {
    gl.destroy();
  }
});
