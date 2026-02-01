import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('checkFramebufferStatus returns FRAMEBUFFER_COMPLETE', async () => {
  const gl = await webGL2();
  try { assert.equal(gl.checkFramebufferStatus(gl.FRAMEBUFFER), gl.FRAMEBUFFER_COMPLETE); } finally { gl.destroy(); }
});
