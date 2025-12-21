import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('scissor does not throw', async () => {
  const gl = await webGL2();
  try {
    gl.scissor(0, 0, 100, 100);
  } finally {
    gl.destroy();
  }
});
