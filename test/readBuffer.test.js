import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('readBuffer state management and readPixels integration', async () => {
    const gl = await webGL2();
    try {
        const fb = gl.createFramebuffer();
        gl.bindFramebuffer(gl.FRAMEBUFFER, fb);

        const tex0 = gl.createTexture();
        gl.bindTexture(gl.TEXTURE_2D, tex0);
        gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA8, 10, 10, 0, gl.RGBA, gl.UNSIGNED_BYTE, null);
        gl.framebufferTexture2D(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.TEXTURE_2D, tex0, 0);

        const tex1 = gl.createTexture();
        gl.bindTexture(gl.TEXTURE_2D, tex1);
        gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA8, 10, 10, 0, gl.RGBA, gl.UNSIGNED_BYTE, null);
        gl.framebufferTexture2D(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT1, gl.TEXTURE_2D, tex1, 0);

        // Clear Attachment 0 to Red
        gl.drawBuffers([gl.COLOR_ATTACHMENT0]);
        gl.clearColor(1, 0, 0, 1);
        gl.clear(gl.COLOR_BUFFER_BIT);

        // Clear Attachment 1 to Green
        gl.drawBuffers([gl.NONE, gl.COLOR_ATTACHMENT1]);
        gl.clearColor(0, 1, 0, 1);
        gl.clear(gl.COLOR_BUFFER_BIT);

        const pixels = new Uint8Array(4);

        // Test default readBuffer (COLOR_ATTACHMENT0)
        gl.readPixels(0, 0, 1, 1, gl.RGBA, gl.UNSIGNED_BYTE, pixels);
        console.log('Read from attachment 0 (default):', pixels);
        assert.deepStrictEqual(Array.from(pixels), [255, 0, 0, 255]);

        // Test readBuffer(COLOR_ATTACHMENT1)
        gl.readBuffer(gl.COLOR_ATTACHMENT1);
        gl.readPixels(0, 0, 1, 1, gl.RGBA, gl.UNSIGNED_BYTE, pixels);
        console.log('Read from attachment 1:', pixels);
        assert.deepStrictEqual(Array.from(pixels), [0, 255, 0, 255]);

        // Test readBuffer(COLOR_ATTACHMENT0)
        gl.readBuffer(gl.COLOR_ATTACHMENT0);
        gl.readPixels(0, 0, 1, 1, gl.RGBA, gl.UNSIGNED_BYTE, pixels);
        console.log('Read from attachment 0 (switched back):', pixels);
        assert.deepStrictEqual(Array.from(pixels), [255, 0, 0, 255]);

        // Test readBuffer(BACK) on FBO should fail (INVALID_OPERATION)
        assert.throws(() => gl.readBuffer(gl.BACK), /WASM error 7/);

        // Bind default framebuffer
        gl.bindFramebuffer(gl.FRAMEBUFFER, null);
        
        // Default readBuffer for default FB is BACK (0x0405)
        gl.readBuffer(gl.BACK);
        assert.strictEqual(gl.getError(), 0);

        // COLOR_ATTACHMENT0 on default FB should fail
        assert.throws(() => gl.readBuffer(gl.COLOR_ATTACHMENT0), /WASM error 7/);
    } finally {
        gl.destroy();
    }
});

