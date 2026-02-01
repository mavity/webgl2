import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('WebGL2 smoke test - 1x1 CornflowerBlue round-trip', async () => {
  const gl = await webGL2();
  console.log('Context created');

  try {
    const texHandle = gl.createTexture();
    assert(texHandle && typeof texHandle === 'object' && typeof texHandle._handle === 'number' && texHandle._handle > 0, 'createTexture should return a texture wrapper object');
    console.log(`Texture created (handle: ${texHandle._handle})`);

    gl.bindTexture(0, texHandle);
    console.log('Texture bound');

    // CornflowerBlue: #6495ED = (100, 149, 237, 255)
    const pixelData = new Uint8Array([100, 149, 237, 255]);
    gl.texImage2D(0, 0, 0, 1, 1, 0, 0, 0, pixelData);
    console.log('Pixel data uploaded');

    const fbHandle = gl.createFramebuffer();
    assert(fbHandle && typeof fbHandle === 'object' && typeof fbHandle._handle === 'number' && fbHandle._handle > 0, 'createFramebuffer should return a framebuffer wrapper object');
    console.log(`Framebuffer created (handle: ${fbHandle._handle})`);

    gl.bindFramebuffer(0, fbHandle);
    console.log('Framebuffer bound');

    gl.framebufferTexture2D(0, 0, 0, texHandle, 0);
    console.log('Texture attached to framebuffer');

    const readBuffer = new Uint8Array(4);
    gl.readPixels(0, 0, 1, 1, 0, 0, readBuffer);
    console.log(`Pixels read: r=${readBuffer[0]}, g=${readBuffer[1]}, b=${readBuffer[2]}, a=${readBuffer[3]}`);

    assert.deepStrictEqual(Array.from(readBuffer), [100, 149, 237, 255], 'Pixel data must match CornflowerBlue');
    console.log('Pixel data matches expected CornflowerBlue!');
  } finally {
    // Ensure the context is cleaned up; let errors surface as test failures
    gl.destroy();
    console.log('Context destroyed');
  }
});
