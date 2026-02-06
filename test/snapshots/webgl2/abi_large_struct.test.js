// @ts-check
import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2, getShaderWat } from '../../../index.js';

test('ABI: large struct parameter uses frame allocation (WAT or null)', async () => {
  const gl = await webGL2({ debug: 'rust' });
  try {
    const fs = gl.createShader(gl.FRAGMENT_SHADER);
    gl.shaderSource(fs, `#version 300 es
      precision highp float; out vec4 fragColor;
      struct LargeData { vec4 a; vec4 b; vec4 c; vec4 d; vec4 e; };
      float processLargeData(LargeData data) { return data.a.x + data.b.y + data.c.z + data.d.w + data.e.x; }
      void main() { LargeData l; fragColor = vec4(processLargeData(l),0.0,0.0,1.0);} `);
    gl.compileShader(fs);

    const vs = gl.createShader(gl.VERTEX_SHADER);
    gl.shaderSource(vs, `#version 300 es
      void main() { gl_Position = vec4(0); }`);
    gl.compileShader(vs);

    const program = gl.createProgram();
    gl.attachShader(program, vs);
    gl.attachShader(program, fs);
    gl.linkProgram(program);
    assert.strictEqual(gl.getProgramParameter(program, gl.LINK_STATUS), true);

    const fsWat = getShaderWat(gl._ctxHandle, program._handle, gl.FRAGMENT_SHADER);
    assert.ok(fsWat.includes('f32.load'), 'Should load data member');
    assert.ok(fsWat.includes('i32.const 20'), 'Should have offset for b.y');
    assert.ok(fsWat.includes('i32.const 60'), 'Should have offset for d.w');
  } finally {
    gl.destroy();
  }
});

test('ABI: nested large struct (WAT or null)', async () => {
  const gl = await webGL2({ debug: 'rust' });
  try {
    const fs = gl.createShader(gl.FRAGMENT_SHADER);
    gl.shaderSource(fs, `#version 300 es
      precision highp float; out vec4 fragColor;
      struct Inner { vec4 a; vec4 b; };
      struct Outer { Inner i1; Inner i2; vec4 extra; };
      float processNested(Outer o) { return o.i1.a.x + o.i2.b.y + o.extra.z; }
      void main() { Outer o; fragColor = vec4(processNested(o),0.0,0.0,1.0);} `);
    gl.compileShader(fs);

    const vs = gl.createShader(gl.VERTEX_SHADER);
    gl.shaderSource(vs, `#version 300 es
      void main() { gl_Position = vec4(0); }`);
    gl.compileShader(vs);

    const program = gl.createProgram();
    gl.attachShader(program, vs);
    gl.attachShader(program, fs);
    gl.linkProgram(program);
    assert.strictEqual(gl.getProgramParameter(program, gl.LINK_STATUS), true);

    const fsWat = getShaderWat(gl._ctxHandle, program._handle, gl.FRAGMENT_SHADER);
    assert.ok(fsWat.includes('f32.load'), 'Should load data member');
    assert.ok(fsWat.includes('i32.const 52'), 'Should load o.i2.b.y');
    assert.ok(fsWat.includes('i32.const 72'), 'Should load o.extra.z');
  } finally {
    gl.destroy();
  }
});
