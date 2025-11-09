import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('deleteQuery throws not implemented', async () => {
  const gl = await webGL2();
  try { assert.throws(() => gl.deleteQuery(1), /not implemented/); } finally { gl.destroy(); }
});
