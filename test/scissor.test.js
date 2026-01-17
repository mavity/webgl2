import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('scissor test affects clearing', async () => {
  const gl = await webGL2(2, 2);
  try {
    gl.clearColor(1, 0, 0, 1); // Red
    gl.enable(gl.SCISSOR_TEST);
    gl.scissor(0, 0, 1, 1); // Only bottom-left quadrant (in WebGL Y is up, but our implementation currently has some conventions to check)
    // Wait, let's just clear a 1x1 area and see where it lands.
    gl.clear(gl.COLOR_BUFFER_BIT);

    const pixels = new Uint8Array(16);
    gl.readPixels(0, 0, 2, 2, gl.RGBA, gl.UNSIGNED_BYTE, pixels);

    // One pixel should be red, others zero.
    // In our current software rasterizer, (0,0) is top-left or bottom-left?
    // viewport(0,0,w,h) maps -1..1 to 0..w, 0..h.
    // gl.scissor usually follows viewport coordinates.
    
    let count = 0;
    for (let i = 0; i < 4; i++) {
        if (pixels[i*4] === 255) count++;
    }
    assert.equal(count, 1, "Exactly one pixel should be red");
  } finally {
    gl.destroy();
  }
});
