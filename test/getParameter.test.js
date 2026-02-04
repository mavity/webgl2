import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('getParameter returns viewport and clear color', async () => {
  const gl = await webGL2();
  try {
    const viewport = gl.getParameter(gl.VIEWPORT);
    assert.ok(viewport instanceof Int32Array);
    assert.strictEqual(viewport.length, 4);
    
    const clearColor = gl.getParameter(gl.COLOR_CLEAR_VALUE);
    assert.ok(clearColor instanceof Float32Array);
    assert.strictEqual(clearColor.length, 4);

    const maxDrawBuffers = gl.getParameter(gl.MAX_DRAW_BUFFERS);
    assert.strictEqual(maxDrawBuffers, 8);

    const maxColorAttachments = gl.getParameter(gl.MAX_COLOR_ATTACHMENTS);
    assert.strictEqual(maxColorAttachments, 8);

    const fb = gl.createFramebuffer();
    gl.bindFramebuffer(gl.FRAMEBUFFER, fb);
    const boundFB = gl.getParameter(gl.FRAMEBUFFER_BINDING);
    assert.strictEqual(boundFB, fb);

    gl.drawBuffers([gl.COLOR_ATTACHMENT0, gl.COLOR_ATTACHMENT1]);
    const drawBuf0 = gl.getParameter(gl.DRAW_BUFFER0);
    assert.strictEqual(drawBuf0, gl.COLOR_ATTACHMENT0);
    const drawBuf1 = gl.getParameter(gl.DRAW_BUFFER1);
    assert.strictEqual(drawBuf1, gl.COLOR_ATTACHMENT1);
  } finally {
    gl.destroy();
  }
});
