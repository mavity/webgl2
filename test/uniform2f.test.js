import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('uniform2f does not throw', async () => {
  const gl = await webGL2();
  try { gl.uniform2f(null, 0.0, 0.0); } finally { gl.destroy(); }
});
