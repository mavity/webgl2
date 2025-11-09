import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('bufferSubData throws not implemented', async () => {
  const gl = await webGL2();
  try { assert.throws(() => gl.bufferSubData(0, 0, new Uint8Array([1])), /not implemented/); } finally { gl.destroy(); }
});
