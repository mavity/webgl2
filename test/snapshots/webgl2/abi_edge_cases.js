// @ts-check
// ABI Test: Edge cases and boundary conditions
// Tests threshold boundaries, empty structs, and unusual combinations

import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2, getShaderWat } from '../../../index.js';

test('ABI: exactly 16 byte struct (at threshold)', async () => {
  const gl = await webGL2({ debug: 'rust' });

  try {
    const fs = gl.createShader(gl.FRAGMENT_SHADER);
    gl.shaderSource(fs, `#version 300 es
      precision highp float;
      out vec4 fragColor;
      
      struct ExactlyThreshold {
        vec4 data;
      };
      
      float processThreshold(ExactlyThreshold s) {
        return s.data.x + s.data.y + s.data.z + s.data.w;
      }
      
      void main() {
        ExactlyThreshold s;
        s.data = vec4(1.0, 2.0, 3.0, 4.0);
        float result = processThreshold(s);
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
    assert.strictEqual(status, true, 'Program with 16-byte struct should link successfully');

    const fsWat = getShaderWat(gl._ctxHandle, program._handle, gl.FRAGMENT_SHADER);
    
    // Exactly 16 bytes should be flattened (at threshold)
    console.log('16-byte struct WAT preview (first 1200 chars):');
    console.log(fsWat.substring(0, 1200));
    
  } finally {
    gl.destroy();
  }
});

test('ABI: 17 byte struct (just over threshold)', async () => {
  const gl = await webGL2({ debug: 'rust' });

  try {
    const fs = gl.createShader(gl.FRAGMENT_SHADER);
    gl.shaderSource(fs, `#version 300 es
      precision highp float;
      out vec4 fragColor;
      
      struct JustOverThreshold {
        vec4 data;
        float extra;
      };
      
      float processOver(JustOverThreshold s) {
        return s.data.x + s.extra;
      }
      
      void main() {
        JustOverThreshold s;
        s.data = vec4(1.0, 2.0, 3.0, 4.0);
        s.extra = 5.0;
        float result = processOver(s);
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
    assert.strictEqual(status, true, 'Program with 20-byte struct should link successfully');

    const fsWat = getShaderWat(gl._ctxHandle, program._handle, gl.FRAGMENT_SHADER);
    
    // Just over 16 bytes should use frame
    assert.match(fsWat, /global\.get.*5/, 'Should use frame for >16 byte struct');
    assert.match(fsWat, /\(type.*\(func \(param i32\)/, 'Should take i32 pointer');
    
  } finally {
    gl.destroy();
  }
});

test('ABI: struct with mixed types', async () => {
  const gl = await webGL2({ debug: 'rust' });

  try {
    const fs = gl.createShader(gl.FRAGMENT_SHADER);
    gl.shaderSource(fs, `#version 300 es
      precision highp float;
      out vec4 fragColor;
      
      struct MixedTypes {
        float f;
        int i;
        vec2 v;
        bool b;
      };
      
      float processMixed(MixedTypes m) {
        return m.b ? (m.f + float(m.i) + m.v.x + m.v.y) : 0.0;
      }
      
      void main() {
        MixedTypes m;
        m.f = 1.0;
        m.i = 2;
        m.v = vec2(3.0, 4.0);
        m.b = true;
        float result = processMixed(m);
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
    assert.strictEqual(status, true, 'Program with mixed-type struct should link successfully');
    
  } finally {
    gl.destroy();
  }
});

test('ABI: array of structs', async () => {
  const gl = await webGL2({ debug: 'rust' });

  try {
    const fs = gl.createShader(gl.FRAGMENT_SHADER);
    gl.shaderSource(fs, `#version 300 es
      precision highp float;
      out vec4 fragColor;
      
      struct Point {
        vec2 pos;
      };
      
      float sumPoints(Point points[5]) {
        float sum = 0.0;
        for (int i = 0; i < 5; i++) {
          sum += points[i].pos.x + points[i].pos.y;
        }
        return sum;
      }
      
      void main() {
        Point points[5];
        for (int i = 0; i < 5; i++) {
          points[i].pos = vec2(float(i), float(i * 2));
        }
        float result = sumPoints(points);
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
    assert.strictEqual(status, true, 'Program with array of structs should link successfully');

    const fsWat = getShaderWat(gl._ctxHandle, program._handle, gl.FRAGMENT_SHADER);
    
    // Array of 5 Point structs (5 * 8 = 40 bytes) should use frame
    assert.match(fsWat, /global\.get.*5/, 'Should use frame for array of structs');
    
  } finally {
    gl.destroy();
  }
});

test('ABI: deeply nested struct', async () => {
  const gl = await webGL2({ debug: 'rust' });

  try {
    const fs = gl.createShader(gl.FRAGMENT_SHADER);
    gl.shaderSource(fs, `#version 300 es
      precision highp float;
      out vec4 fragColor;
      
      struct Level3 {
        vec2 data;
      };
      
      struct Level2 {
        Level3 inner;
        float value;
      };
      
      struct Level1 {
        Level2 inner;
        vec2 extra;
      };
      
      struct Level0 {
        Level1 inner;
        vec2 more;
      };
      
      float processNested(Level0 data) {
        return data.inner.inner.inner.data.x + 
               data.inner.inner.value + 
               data.inner.extra.y + 
               data.more.x;
      }
      
      void main() {
        Level0 data;
        data.inner.inner.inner.data = vec2(1.0, 2.0);
        data.inner.inner.value = 3.0;
        data.inner.extra = vec2(4.0, 5.0);
        data.more = vec2(6.0, 7.0);
        
        float result = processNested(data);
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
    assert.strictEqual(status, true, 'Program with deeply nested struct should link successfully');

    const fsWat = getShaderWat(gl._ctxHandle, program._handle, gl.FRAGMENT_SHADER);
    
    // Deeply nested struct should likely use frame
    console.log('Deeply nested struct WAT preview (first 1500 chars):');
    console.log(fsWat.substring(0, 1500));
    
  } finally {
    gl.destroy();
  }
});
