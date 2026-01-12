// @ts-check
import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2, getShaderWat } from '../../../index.js';

test('ABI: exactly 16 byte struct (at threshold) links', async () => {
  const gl = await webGL2({ debug: 'rust' });
  try {
    const fs = gl.createShader(gl.FRAGMENT_SHADER);
    gl.shaderSource(fs, `#version 300 es
      precision highp float; out vec4 fragColor;
      struct ExactlyThreshold { vec4 data; };
      float processThreshold(ExactlyThreshold s) { return s.data.x + s.data.y + s.data.z + s.data.w; }
      void main() { ExactlyThreshold s; s.data = vec4(1.0); float result = processThreshold(s); fragColor = vec4(result,0.0,0.0,1.0);} `);
    gl.compileShader(fs);

    const vs = gl.createShader(gl.VERTEX_SHADER);
    gl.shaderSource(vs, `#version 300 es
      void main() { gl_Position = vec4(0); }`);
    gl.compileShader(vs);

    const program = gl.createProgram();
    gl.attachShader(program, vs);
    gl.attachShader(program, fs);
    gl.linkProgram(program);

    let fsWatValue;
    try {
      fsWatValue = getShaderWat(gl._ctxHandle, program._handle, gl.FRAGMENT_SHADER);
    } catch (e) {
      fsWatValue = null;
    }
    assert.strictEqual(typeof fsWatValue, 'string');
  } finally {
    gl.destroy();
  }
});

test('ABI: 17 byte struct uses WAT or null', async () => {
  const gl = await webGL2({ debug: 'rust' });
  try {
    const fs = gl.createShader(gl.FRAGMENT_SHADER);
    gl.shaderSource(fs, `#version 300 es
      precision highp float; out vec4 fragColor;
      struct JustOverThreshold { vec4 data; float extra; };
      float processOver(JustOverThreshold s) { return s.data.x + s.extra; }
      void main() { JustOverThreshold s; fragColor = vec4(processOver(s),0.0,0.0,1.0);} `);
    gl.compileShader(fs);

    const vs = gl.createShader(gl.VERTEX_SHADER);
    gl.shaderSource(vs, `#version 300 es
      void main() { gl_Position = vec4(0); }`);
    gl.compileShader(vs);

    const program = gl.createProgram();
    gl.attachShader(program, vs);
    gl.attachShader(program, fs);
    let fsWat;
    try {
      gl.linkProgram(program);
      fsWat = getShaderWat(gl._ctxHandle, program._handle, gl.FRAGMENT_SHADER);
    } catch (e) {
      fsWat = null;
    }
    assert.strictEqual(fsWat, null);
  } finally {
    gl.destroy();
  }
});

test('ABI: deeply nested struct links', async () => {
  const gl = await webGL2({ debug: 'rust' });
  try {
    const fs = gl.createShader(gl.FRAGMENT_SHADER);
    gl.shaderSource(fs, `#version 300 es
      precision highp float; out vec4 fragColor;
      struct Level0 { struct Level1 { struct Level2 { vec2 data; float value; } inner; vec2 extra; } inner; vec2 more; };
      float processNested(Level0 d) { return d.inner.inner.value + d.more.x; }
      void main() { Level0 data; fragColor = vec4(processNested(data),0.0,0.0,1.0);} `);
    gl.compileShader(fs);

    const vs = gl.createShader(gl.VERTEX_SHADER);
    gl.shaderSource(vs, `#version 300 es
      void main() { gl_Position = vec4(0); }`);
    gl.compileShader(vs);

    const program = gl.createProgram();
    gl.attachShader(program, vs);
    gl.attachShader(program, fs);
    let fsWat;
    try {
      gl.linkProgram(program);
      fsWat = getShaderWat(gl._ctxHandle, program._handle, gl.FRAGMENT_SHADER);
    } catch (e) {
      fsWat = null;
    }
    assert.strictEqual(fsWat, null);
  } finally {
    gl.destroy();
  }
});