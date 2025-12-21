import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('disableVertexAttribArray does not throw', async () => {
  const gl = await webGL2();
  try { gl.disableVertexAttribArray(0); } finally { gl.destroy(); }
});
