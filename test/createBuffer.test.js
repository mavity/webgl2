import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('createBuffer returns a buffer object', async () => {
  const gl = await webGL2();
  try {
    const buffer = gl.createBuffer();
    assert.ok(buffer);
  } finally { gl.destroy(); }
});
