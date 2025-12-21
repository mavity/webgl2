import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('uniform1i does not throw', async () => {
  const gl = await webGL2();
  try { gl.uniform1i(null, 1); } finally { gl.destroy(); }
});
