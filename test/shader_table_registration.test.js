import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('Shaders are registered in function table', async () => {
  const gl = await webGL2();
  
  const vs = gl.createShader(gl.VERTEX_SHADER);
  gl.shaderSource(vs, '#version 300 es\nvoid main() { gl_Position = vec4(0); }');
  gl.compileShader(vs);
  
  const fs = gl.createShader(gl.FRAGMENT_SHADER);
  gl.shaderSource(fs, '#version 300 es\nprecision mediump float; out vec4 color; void main() { color = vec4(1); }');
  gl.compileShader(fs);
  
  const prog = gl.createProgram();
  gl.attachShader(prog, vs);
  gl.attachShader(prog, fs);
  gl.linkProgram(prog);
  
  assert.ok(prog._vsTableIndex !== undefined, 'VS should have table index');
  assert.ok(prog._fsTableIndex !== undefined, 'FS should have table index');
  assert.ok(prog._vsTableIndex > 0, 'VS index should be positive');
  assert.ok(prog._fsTableIndex > 0, 'FS index should be positive');
  
  const vsFunc = gl._sharedTable.get(prog._vsTableIndex);
  const fsFunc = gl._sharedTable.get(prog._fsTableIndex);
  assert.strictEqual(typeof vsFunc, 'function', 'VS should be callable');
  assert.strictEqual(typeof fsFunc, 'function', 'FS should be callable');
  
  gl.destroy();
});
