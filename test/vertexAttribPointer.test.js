import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('vertexAttribPointer throws not implemented', async () => {
  const gl = await webGL2();
  try { assert.throws(() => gl.vertexAttribPointer(0, 3, 0, false, 0, 0), /not implemented/); } finally { gl.destroy(); }
});
