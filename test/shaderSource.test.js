import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('shaderSource throws not implemented', async () => {
  const gl = await webGL2();
  try { assert.throws(() => gl.shaderSource(1, 'src'), /not implemented/); } finally { gl.destroy(); }
});
