import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('uniform3f does not throw', async () => {
  const gl = await webGL2();
  try { gl.uniform3f(null, 0.0, 0.0, 0.0); } finally { gl.destroy(); }
});
