import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('drawBuffers on default framebuffer (GL_BACK)', async () => {
    const gl = await webGL2({ width: 10, height: 10 });
    try {
        // Default framebuffer only supports [GL_BACK] or [GL_NONE]
        gl.drawBuffers([gl.BACK]);
        assert.strictEqual(gl.getError(), 0);

        gl.drawBuffers([0]); // GL_NONE
        assert.strictEqual(gl.getError(), 0);

        // This should fail
        assert.throws(() => gl.drawBuffers([gl.COLOR_ATTACHMENT0]), /WASM error 7/);
    } finally {
        gl.destroy();
    }
});

test('drawBuffers on FBO', async () => {
    const gl = await webGL2({ width: 10, height: 10 });
    try {
        const fb = gl.createFramebuffer();
        gl.bindFramebuffer(gl.FRAMEBUFFER, fb);

        // Default for FBO is [GL_COLOR_ATTACHMENT0, GL_NONE, ...]
        gl.drawBuffers([gl.COLOR_ATTACHMENT0, gl.COLOR_ATTACHMENT1]);
        assert.strictEqual(gl.getError(), 0);

        // Invalid: wrong order or non-matching index
        assert.throws(() => gl.drawBuffers([gl.COLOR_ATTACHMENT1, gl.COLOR_ATTACHMENT0]), /WASM error 7/);

        gl.drawBuffers([gl.NONE, gl.COLOR_ATTACHMENT1]);
        assert.strictEqual(gl.getError(), 0);
    } finally {
        gl.destroy();
    }
});
