import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('vertexAttrib4f sets default value', async () => {
  const gl = await webGL2();
  try {
    // This should not throw
    gl.vertexAttrib4f(0, 1.0, 2.0, 3.0, 4.0);
    
    // We don't have a way to query it back yet via getVertexAttrib, 
    // but we can verify it doesn't crash and returns ERR_OK.
  } finally {
    gl.destroy();
  }
});

test('vertexAttribPointer binds buffer', async () => {
  const gl = await webGL2();
  try {
    const buffer = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, buffer);
    const data = new Float32Array([1, 2, 3, 4]);
    gl.bufferData(gl.ARRAY_BUFFER, data, gl.STATIC_DRAW);
    
    gl.vertexAttribPointer(0, 4, gl.FLOAT, false, 0, 0);
    gl.enableVertexAttribArray(0);
    
    // Verify drawArrays works with the bound attribute
    const vs = gl.createShader(gl.VERTEX_SHADER);
    gl.shaderSource(vs, '#version 300 es\nlayout(location=0) in vec4 pos; void main() { gl_Position = pos; }');
    gl.compileShader(vs);
    const fs = gl.createShader(gl.FRAGMENT_SHADER);
    gl.shaderSource(fs, '#version 300 es\nprecision mediump float; out vec4 color; void main() { color = vec4(1); }');
    gl.compileShader(fs);
    const program = gl.createProgram();
    gl.attachShader(program, vs);
    gl.attachShader(program, fs);
    gl.linkProgram(program);
    gl.useProgram(program);
    
    gl.drawArrays(gl.TRIANGLES, 0, 1);
  } finally {
    gl.destroy();
  }
});
