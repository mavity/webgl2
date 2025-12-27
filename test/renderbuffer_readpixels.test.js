
import { webGL2 } from '../index.js';
import { test } from 'node:test';
import assert from 'node:assert';

test('readPixels from Renderbuffer (RGBA4)', async () => {
    const gl = await webGL2();
    const fb = gl.createFramebuffer();
    gl.bindFramebuffer(gl.FRAMEBUFFER, fb);

    const rb = gl.createRenderbuffer();
    gl.bindRenderbuffer(gl.RENDERBUFFER, rb);
    gl.renderbufferStorage(gl.RENDERBUFFER, gl.RGBA4, 2, 2);
    gl.framebufferRenderbuffer(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.RENDERBUFFER, rb);

    // Clear to Red
    gl.clearColor(1, 0, 0, 1);
    gl.clear(gl.COLOR_BUFFER_BIT);

    const pixels = new Uint8Array(4 * 4); // 2x2 * 4 bytes (RGBA)
    gl.readPixels(0, 0, 2, 2, gl.RGBA, gl.UNSIGNED_BYTE, pixels);

    // Check first pixel
    assert.strictEqual(pixels[0], 255, 'Red channel should be 255');
    assert.strictEqual(pixels[1], 0, 'Green channel should be 0');
    assert.strictEqual(pixels[2], 0, 'Blue channel should be 0');
    assert.strictEqual(pixels[3], 255, 'Alpha channel should be 255');

    gl.deleteRenderbuffer(rb);
    gl.deleteFramebuffer(fb);
    gl.destroy();
});

test('readPixels from Renderbuffer (RGB565)', async () => {
    const gl = await webGL2();
    const fb = gl.createFramebuffer();
    gl.bindFramebuffer(gl.FRAMEBUFFER, fb);

    const rb = gl.createRenderbuffer();
    gl.bindRenderbuffer(gl.RENDERBUFFER, rb);
    gl.renderbufferStorage(gl.RENDERBUFFER, gl.RGB565, 2, 2);
    gl.framebufferRenderbuffer(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.RENDERBUFFER, rb);

    // Clear to Green
    gl.clearColor(0, 1, 0, 1);
    gl.clear(gl.COLOR_BUFFER_BIT);

    const pixels = new Uint8Array(4 * 4); // 2x2 * 4 bytes (RGBA)
    gl.readPixels(0, 0, 2, 2, gl.RGBA, gl.UNSIGNED_BYTE, pixels);

    // Check first pixel
    assert.strictEqual(pixels[0], 0, 'Red channel should be 0');
    assert.strictEqual(pixels[1], 255, 'Green channel should be 255');
    assert.strictEqual(pixels[2], 0, 'Blue channel should be 0');
    assert.strictEqual(pixels[3], 255, 'Alpha channel should be 255');

    gl.deleteRenderbuffer(rb);
    gl.deleteFramebuffer(fb);
    gl.destroy();
});

test('readPixels from Renderbuffer (RGB5_A1)', async () => {
    const gl = await webGL2();
    const fb = gl.createFramebuffer();
    gl.bindFramebuffer(gl.FRAMEBUFFER, fb);

    const rb = gl.createRenderbuffer();
    gl.bindRenderbuffer(gl.RENDERBUFFER, rb);
    gl.renderbufferStorage(gl.RENDERBUFFER, gl.RGB5_A1, 2, 2);
    gl.framebufferRenderbuffer(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.RENDERBUFFER, rb);

    // Clear to Blue with Alpha 1.0
    gl.clearColor(0, 0, 1, 1);
    gl.clear(gl.COLOR_BUFFER_BIT);

    const pixels = new Uint8Array(4 * 4); // 2x2 * 4 bytes (RGBA)
    gl.readPixels(0, 0, 2, 2, gl.RGBA, gl.UNSIGNED_BYTE, pixels);

    // Check first pixel
    assert.strictEqual(pixels[0], 0, 'Red channel should be 0');
    assert.strictEqual(pixels[1], 0, 'Green channel should be 0');
    assert.strictEqual(pixels[2], 255, 'Blue channel should be 255');
    assert.strictEqual(pixels[3], 255, 'Alpha channel should be 255');

    // Clear to Blue with Alpha 0.0
    // Note: RGB5_A1 only supports 1-bit alpha (0 or 1). 0.0 -> 0.
    gl.clearColor(0, 0, 1, 0);
    gl.clear(gl.COLOR_BUFFER_BIT);
    gl.readPixels(0, 0, 2, 2, gl.RGBA, gl.UNSIGNED_BYTE, pixels);

    assert.strictEqual(pixels[3], 0, 'Alpha channel should be 0');

    gl.deleteRenderbuffer(rb);
    gl.deleteFramebuffer(fb);
    gl.destroy();
});
