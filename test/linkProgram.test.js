import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('linkProgram sets LINK_STATUS to true for valid program', async () => {
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
    assert.strictEqual(status, true, 'LINK_STATUS should be true');
  } finally { gl.destroy(); }
});

test('linkProgram sets LINK_STATUS to false for mismatched shaders', async () => {
  const gl = await webGL2();
  try {
    const vs = gl.createShader(gl.VERTEX_SHADER);
    gl.shaderSource(vs, '#version 300 es\nvoid main() { gl_Position = vec4(0); }');
    gl.compileShader(vs);
    // No fragment shader attached
    
    const program = gl.createProgram();
    gl.attachShader(program, vs);
    gl.linkProgram(program);
    
    const status = gl.getProgramParameter(program, gl.LINK_STATUS);
    assert.strictEqual(status, false, 'LINK_STATUS should be false');
    const log = gl.getProgramInfoLog(program);
    assert.ok(log.length > 0, 'Program info log should not be empty');
  } finally { gl.destroy(); }
});
