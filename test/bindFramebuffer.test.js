import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('bindFramebuffer accepts a target and handle', async () => {
  const gl = await webGL2();
  try {
    const fb = gl.createFramebuffer();
    assert(fb && typeof fb === 'object');
    gl.bindFramebuffer(gl.FRAMEBUFFER, fb);
  } finally {
    gl.destroy();
  }
});
