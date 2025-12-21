import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('uniformMatrix4fv does not throw', async () => {
  const gl = await webGL2();
  try { gl.uniformMatrix4fv(null, false, new Float32Array(16)); } finally { gl.destroy(); }
});
