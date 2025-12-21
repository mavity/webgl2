import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('deleteBuffer does not throw', async () => {
  const gl = await webGL2();
  try {
    const buffer = gl.createBuffer();
    gl.deleteBuffer(buffer);
  } finally { gl.destroy(); }
});
