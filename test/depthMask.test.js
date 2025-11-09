import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('depthMask throws not implemented', async () => {
  const gl = await webGL2();
  try { assert.throws(() => gl.depthMask(true), /not implemented/); } finally { gl.destroy(); }
});
