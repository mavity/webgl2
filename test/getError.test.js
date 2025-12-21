import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('getError returns NO_ERROR initially', async () => {
  const gl = await webGL2();
  try {
    const error = gl.getError();
    assert.strictEqual(error, gl.NO_ERROR, 'Initial error should be NO_ERROR');
  } finally { gl.destroy(); }
});
