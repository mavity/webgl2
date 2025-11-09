import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('blendFunc throws not implemented', async () => {
  const gl = await webGL2();
  try { assert.throws(() => gl.blendFunc(0,0), /not implemented/); } finally { gl.destroy(); }
});
