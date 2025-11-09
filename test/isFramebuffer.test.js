import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('isFramebuffer throws not implemented', async () => {
  const gl = await webGL2();
  try { assert.throws(() => gl.isFramebuffer(1), /not implemented/); } finally { gl.destroy(); }
});
