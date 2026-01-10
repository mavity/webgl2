// @ts-check
// ABI Test: Nested function calls with frame allocation
// Tests that frame allocation is properly isolated across nested calls

import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2, getShaderWat } from '../../../index.js';

test('ABI: nested calls with frame isolation', async () => {
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
      
      float innerProcess(LargeData data) {
        return data.a.x + data.b.y + data.c.z;
      }
      
      float outerProcess(LargeData data1, LargeData data2) {
        float r1 = innerProcess(data1);
        float r2 = innerProcess(data2);
        return r1 + r2;
      }
      
      void main() {
        LargeData d1;
        d1.a = vec4(1.0, 2.0, 3.0, 4.0);
        d1.b = vec4(5.0, 6.0, 7.0, 8.0);
        d1.c = vec4(9.0, 10.0, 11.0, 12.0);
        
        LargeData d2;
        d2.a = vec4(13.0, 14.0, 15.0, 16.0);
        d2.b = vec4(17.0, 18.0, 19.0, 20.0);
        d2.c = vec4(21.0, 22.0, 23.0, 24.0);
        
        float result = outerProcess(d1, d2);
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
    assert.strictEqual(status, true, 'Program with nested calls should link successfully');

    const fsWat = getShaderWat(gl._ctxHandle, program._handle, gl.FRAGMENT_SHADER);
    
    // Should have multiple frame allocations and deallocations
    const frameGets = (fsWat.match(/global\.get 5/g) || []).length;
    const frameSets = (fsWat.match(/global\.set 5/g) || []).length;
    
    assert.ok(frameGets >= 2, `Should have at least 2 frame gets (nested calls), got ${frameGets}`);
    assert.ok(frameSets >= 2, `Should have at least 2 frame sets (nested calls), got ${frameSets}`);
    
    console.log(`Frame operations: ${frameGets} gets, ${frameSets} sets`);
    console.log('Nested calls WAT preview (first 2500 chars):');
    console.log(fsWat.substring(0, 2500));
    
  } finally {
    gl.destroy();
  }
});

test('ABI: recursive-style call pattern', async () => {
  const gl = await webGL2({ debug: 'rust' });

  try {
    const fs = gl.createShader(gl.FRAGMENT_SHADER);
    gl.shaderSource(fs, `#version 300 es
      precision highp float;
      out vec4 fragColor;
      
      struct Data {
        vec4 a;
        vec4 b;
        vec4 c;
      };
      
      float processData(Data data, int depth) {
        if (depth <= 0) {
          return data.a.x;
        }
        
        Data modified;
        modified.a = data.b;
        modified.b = data.c;
        modified.c = data.a;
        
        return processData(modified, depth - 1) + data.b.y;
      }
      
      void main() {
        Data d;
        d.a = vec4(1.0, 2.0, 3.0, 4.0);
        d.b = vec4(5.0, 6.0, 7.0, 8.0);
        d.c = vec4(9.0, 10.0, 11.0, 12.0);
        
        float result = processData(d, 2);
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
    assert.strictEqual(status, true, 'Program with recursive-style calls should link successfully');

    const fsWat = getShaderWat(gl._ctxHandle, program._handle, gl.FRAGMENT_SHADER);
    
    // Should handle frame allocation properly even with potential recursion
    assert.match(fsWat, /global\.get.*5/, 'Should use frame allocation');
    assert.match(fsWat, /\(type.*\(func \(param i32 i32\)/, 
      'processData should take i32 pointer and int depth');
    
  } finally {
    gl.destroy();
  }
});
