import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('getBufferParameter returns buffer size', async () => {
  const gl = await webGL2();
  try {
    const buffer = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, buffer);
    const data = new Uint8Array([1, 2, 3, 4]);
    gl.bufferData(gl.ARRAY_BUFFER, data, gl.STATIC_DRAW);
    
    const size = gl.getBufferParameter(gl.ARRAY_BUFFER, gl.BUFFER_SIZE);
    assert.strictEqual(size, 4);
  } finally {
    gl.destroy();
  }
});
