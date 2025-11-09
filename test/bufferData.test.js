import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('bufferData throws not implemented', async () => {
  const gl = await webGL2();
  try { assert.throws(() => gl.bufferData(0, new Uint8Array([1,2]), 0), /not implemented/); } finally { gl.destroy(); }
});
