import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('uniformMatrix4fv throws not implemented', async () => {
  const gl = await webGL2();
  try { assert.throws(() => gl.uniformMatrix4fv(0, false, new Float32Array(16)), /not implemented/); } finally { gl.destroy(); }
});
