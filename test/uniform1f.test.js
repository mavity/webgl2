import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('uniform1f does not throw', async () => {
  const gl = await webGL2();
  try { gl.uniform1f(null, 0.0); } finally { gl.destroy(); }
});
