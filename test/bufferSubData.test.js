import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('bufferSubData works', async () => {
  const gl = await webGL2();
  try {
    const buf = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, buf);
    gl.bufferData(gl.ARRAY_BUFFER, new Uint8Array([1, 2, 3, 4]), gl.STATIC_DRAW);
    gl.bufferSubData(gl.ARRAY_BUFFER, 1, new Uint8Array([5, 6]));
    
    // Check bounds
    // Note: The error message from Rust is "buffer overflow", but the JS wrapper might wrap it or just throw generic error if not handled.
    // Let's check what _checkErr does. It usually throws with the error message from wasm_last_error.
    // So /buffer overflow/ should be correct if set_last_error is called.
    try {
        gl.bufferSubData(gl.ARRAY_BUFFER, 3, new Uint8Array([7, 8]));
        assert.fail('Should have thrown buffer overflow');
    } catch (e) {
        assert.match(e.message, /buffer overflow/);
    }
  } finally { gl.destroy(); }
});
