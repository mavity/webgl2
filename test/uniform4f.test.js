import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('uniform4f throws not implemented', async () => {
  const gl = await webGL2();
  try { assert.throws(() => gl.uniform4f(0, 0,0,0,0), /not implemented/); } finally { gl.destroy(); }
});
