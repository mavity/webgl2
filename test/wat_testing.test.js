import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2, getShaderModule, getShaderWat } from '../index.js';

test('getShaderModule returns WASM bytes for compiled vertex shader', async () => {
  const gl = await webGL2();
  try {
    const vs = gl.createShader(gl.VERTEX_SHADER);
    gl.shaderSource(vs, '#version 300 es\nvoid main() { gl_Position = vec4(0); }');
    gl.compileShader(vs);

    const fs = gl.createShader(gl.FRAGMENT_SHADER);
    gl.shaderSource(fs, '#version 300 es\nprecision mediump float; out vec4 color; void main() { color = vec4(1); }');
    gl.compileShader(fs);

    const program = gl.createProgram();
    gl.attachShader(program, vs);
    gl.attachShader(program, fs);
    gl.linkProgram(program);

    const status = gl.getProgramParameter(program, gl.LINK_STATUS);
    assert.strictEqual(status, true, 'Program should link successfully');

    // Get WASM bytes for vertex shader - use program._handle to get the numeric handle
    const wasmBytes = getShaderModule(gl._ctxHandle, program._handle, gl.VERTEX_SHADER);

    const firstFourBytes = [...wasmBytes.slice(0, 4)];
    assert.deepEqual(firstFourBytes, [0x00, 0x61, 0x73, 0x6D], 'WASM magic number should match');
  } finally {
    gl.destroy();
  }
});

test('getShaderModule returns WASM bytes for compiled fragment shader', async () => {
  const gl = await webGL2();
  try {
    const vs = gl.createShader(gl.VERTEX_SHADER);
    gl.shaderSource(vs, '#version 300 es\nvoid main() { gl_Position = vec4(0); }');
    gl.compileShader(vs);

    const fs = gl.createShader(gl.FRAGMENT_SHADER);
    gl.shaderSource(fs, '#version 300 es\nprecision mediump float; out vec4 color; void main() { color = vec4(1); }');
    gl.compileShader(fs);

    const program = gl.createProgram();
    gl.attachShader(program, vs);
    gl.attachShader(program, fs);
    gl.linkProgram(program);

    const status = gl.getProgramParameter(program, gl.LINK_STATUS);
    assert.strictEqual(status, true, 'Program should link successfully');

    // Get WASM bytes for fragment shader - use program._handle to get the numeric handle
    const wasmBytes = getShaderModule(gl._ctxHandle, program._handle, gl.FRAGMENT_SHADER);

    const firstFourBytes = [...wasmBytes.slice(0, 4)];
    assert.deepEqual(firstFourBytes, [0x00, 0x61, 0x73, 0x6D], 'WASM magic number should match');
  } finally {
    gl.destroy();
  }
});

test('getShaderModule returns null for unlinked program', async () => {
  const gl = await webGL2();
  try {
    const program = gl.createProgram();

    // Get WASM bytes before linking - use program._handle to get the numeric handle
    const wasmBytes = getShaderModule(gl._ctxHandle, program._handle, gl.VERTEX_SHADER);

    assert.strictEqual(wasmBytes, null, 'WASM bytes should be null for unlinked program');
  } finally {
    gl.destroy();
  }
});

test('getShaderWat returns WAT text for compiled vertex shader', async () => {
  const gl = await webGL2();
  try {
    const vs = gl.createShader(gl.VERTEX_SHADER);
    gl.shaderSource(vs, '#version 300 es\nvoid main() { gl_Position = vec4(0); }');
    gl.compileShader(vs);

    const fs = gl.createShader(gl.FRAGMENT_SHADER);
    gl.shaderSource(fs, '#version 300 es\nprecision mediump float; out vec4 color; void main() { color = vec4(1); }');
    gl.compileShader(fs);

    const program = gl.createProgram();
    gl.attachShader(program, vs);
    gl.attachShader(program, fs);
    gl.linkProgram(program);

    const status = gl.getProgramParameter(program, gl.LINK_STATUS);
    assert.strictEqual(status, true, 'Program should link successfully');

    // Get WAT text for vertex shader - use program._handle to get the numeric handle
    const wat = getShaderWat(gl._ctxHandle, program._handle, gl.VERTEX_SHADER);

    // WAT files start with (module
    assert.ok(wat.includes('(module'), 'WAT should contain (module');
  } finally {
    gl.destroy();
  }
});

test('getShaderWat returns WAT text for compiled fragment shader', async () => {
  const gl = await webGL2();
  try {
    const vs = gl.createShader(gl.VERTEX_SHADER);
    gl.shaderSource(vs, '#version 300 es\nvoid main() { gl_Position = vec4(0); }');
    gl.compileShader(vs);

    const fs = gl.createShader(gl.FRAGMENT_SHADER);
    gl.shaderSource(fs, '#version 300 es\nprecision mediump float; out vec4 color; void main() { color = vec4(1); }');
    gl.compileShader(fs);

    const program = gl.createProgram();
    gl.attachShader(program, vs);
    gl.attachShader(program, fs);
    gl.linkProgram(program);

    const status = gl.getProgramParameter(program, gl.LINK_STATUS);
    assert.strictEqual(status, true, 'Program should link successfully');

    // Get WAT text for fragment shader - use program._handle to get the numeric handle
    const wat = getShaderWat(gl._ctxHandle, program._handle, gl.FRAGMENT_SHADER);

    // WAT files start with (module
    assert.ok(wat.includes('(module'), 'WAT should contain (module');
  } finally {
    gl.destroy();
  }
});

test('getShaderWat returns null for unlinked program', async () => {
  const gl = await webGL2();
  try {
    const program = gl.createProgram();

    // Get WAT text before linking - use program._handle to get the numeric handle
    const wat = getShaderWat(gl._ctxHandle, program._handle, gl.VERTEX_SHADER);

    assert.strictEqual(wat, null, 'WAT text should be null for unlinked program');
  } finally {
    gl.destroy();
  }
});

test('getShaderModule and getShaderWat are consistent', async () => {
  const gl = await webGL2();
  try {
    const vs = gl.createShader(gl.VERTEX_SHADER);
    gl.shaderSource(vs, '#version 300 es\nvoid main() { gl_Position = vec4(0); }');
    gl.compileShader(vs);

    const fs = gl.createShader(gl.FRAGMENT_SHADER);
    gl.shaderSource(fs, '#version 300 es\nprecision mediump float; out vec4 color; void main() { color = vec4(1); }');
    gl.compileShader(fs);

    const program = gl.createProgram();
    gl.attachShader(program, vs);
    gl.attachShader(program, fs);
    gl.linkProgram(program);

    const status = gl.getProgramParameter(program, gl.LINK_STATUS);
    assert.strictEqual(status, true, 'Program should link successfully');

    // Get both WASM and WAT - use program._handle to get the numeric handle
    const wasmBytes = getShaderModule(gl._ctxHandle, program._handle, gl.VERTEX_SHADER);
    const wat = getShaderWat(gl._ctxHandle, program._handle, gl.VERTEX_SHADER);

    // Both should be available
    assert.deepEqual(
      {
        bytesAvailable: wasmBytes !== null,
        watAvailable: wat !== null,
        wasmTextLongerThanBytes: wat && wasmBytes ? wat.length > wasmBytes.length : false
      },
      {
        bytesAvailable: true,
        watAvailable: true,
        wasmTextLongerThanBytes: true
      });
  } finally {
    gl.destroy();
  }
});
