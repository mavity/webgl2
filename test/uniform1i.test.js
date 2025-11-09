import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('uniform1i throws not implemented', async () => {
  const gl = await webGL2();
  try { assert.throws(() => gl.uniform1i(0, 1), /not implemented/); } finally { gl.destroy(); }
});
