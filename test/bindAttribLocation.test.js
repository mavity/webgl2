import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('bindAttribLocation throws not implemented', async () => {
  const gl = await webGL2();
  try { assert.throws(() => gl.bindAttribLocation(1, 0, 'a'), /not implemented/); } finally { gl.destroy(); }
});
