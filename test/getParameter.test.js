import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('getParameter throws not implemented', async () => {
  const gl = await webGL2();
  try { assert.throws(() => gl.getParameter(0), /not implemented/); } finally { gl.destroy(); }
});
