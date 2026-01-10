// @ts-check
// ABI Test: Large struct parameter (>16 bytes) requiring frame allocation
// This test verifies frame-based parameter passing for structures exceeding MAX_FLATTEN_BYTES

import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2, getShaderWat } from '../../../index.js';

test('ABI: large struct parameter uses frame allocation', async () => {
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
        vec4 d;
        vec4 e;
      };
      
      float processLargeData(LargeData data) {
        return data.a.x + data.b.y + data.c.z + data.d.w + data.e.x;
      }
      
      void main() {
        LargeData large;
        large.a = vec4(1.0, 2.0, 3.0, 4.0);
        large.b = vec4(5.0, 6.0, 7.0, 8.0);
        large.c = vec4(9.0, 10.0, 11.0, 12.0);
        large.d = vec4(13.0, 14.0, 15.0, 16.0);
        large.e = vec4(17.0, 18.0, 19.0, 20.0);
        
        float result = processLargeData(large);
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
    assert.strictEqual(status, true, 'Program with large struct should link successfully');

    const fsWat = getShaderWat(gl._ctxHandle, program._handle, gl.FRAGMENT_SHADER);

    // Verify frame allocation patterns in WAT
    assert.match(fsWat, /global\.get.*5/, 'Should get FRAME_SP_GLOBAL (index 5)');
    assert.match(fsWat, /global\.set.*5/, 'Should set FRAME_SP_GLOBAL');
    
    // For large struct (80 bytes), should use i32 pointer parameter
    assert.match(fsWat, /\(type.*\(func \(param i32\)/, 'processLargeData should take i32 pointer parameter');
    
    console.log('Large struct WAT preview (first 2000 chars):');
    console.log(fsWat.substring(0, 2000));
    
  } finally {
    gl.destroy();
  }
});

test('ABI: nested large struct', async () => {
  const gl = await webGL2({ debug: 'rust' });

  try {
    const fs = gl.createShader(gl.FRAGMENT_SHADER);
    gl.shaderSource(fs, `#version 300 es
      precision highp float;
      out vec4 fragColor;
      
      struct Inner {
        vec4 data1;
        vec4 data2;
      };
      
      struct Outer {
        Inner inner1;
        Inner inner2;
        vec4 extra;
      };
      
      float processNested(Outer data) {
        return data.inner1.data1.x + data.inner2.data2.y + data.extra.z;
      }
      
      void main() {
        Outer outer;
        outer.inner1.data1 = vec4(1.0, 2.0, 3.0, 4.0);
        outer.inner1.data2 = vec4(5.0, 6.0, 7.0, 8.0);
        outer.inner2.data1 = vec4(9.0, 10.0, 11.0, 12.0);
        outer.inner2.data2 = vec4(13.0, 14.0, 15.0, 16.0);
        outer.extra = vec4(17.0, 18.0, 19.0, 20.0);
        
        float result = processNested(outer);
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
    assert.strictEqual(status, true, 'Program with nested struct should link successfully');

    const fsWat = getShaderWat(gl._ctxHandle, program._handle, gl.FRAGMENT_SHADER);
    
    // Nested struct is 80 bytes, should use frame
    assert.match(fsWat, /global\.get.*5/, 'Should use frame stack for nested struct');
    
  } finally {
    gl.destroy();
  }
});
