import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('bufferData uploads data and can be queried', async () => {
  const gl = await webGL2();
  try {
    const buffer = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, buffer);
    const data = new Float32Array([1, 2, 3, 4]);
    gl.bufferData(gl.ARRAY_BUFFER, data, gl.STATIC_DRAW);
    
    const size = gl.getBufferParameter(gl.ARRAY_BUFFER, gl.BUFFER_SIZE);
    assert.strictEqual(size, 16, 'Buffer size should be 16 bytes (4 floats)');
    assert.ok(buffer._handle > 0, 'Buffer handle should be valid');
  } finally { gl.destroy(); }
});
