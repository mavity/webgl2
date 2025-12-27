
import { webGL2 } from '../index.js';
import { test } from 'node:test';
import assert from 'node:assert';

test('Renderbuffer Depth/Stencil Attachment', async () => {
    const gl = await webGL2();
    const fb = gl.createFramebuffer();
    gl.bindFramebuffer(gl.FRAMEBUFFER, fb);

    // Test DEPTH_COMPONENT16
    const rbDepth = gl.createRenderbuffer();
    gl.bindRenderbuffer(gl.RENDERBUFFER, rbDepth);
    gl.renderbufferStorage(gl.RENDERBUFFER, gl.DEPTH_COMPONENT16, 2, 2);
    gl.framebufferRenderbuffer(gl.FRAMEBUFFER, gl.DEPTH_ATTACHMENT, gl.RENDERBUFFER, rbDepth);
    
    assert.strictEqual(gl.getError(), gl.NO_ERROR, 'Should be no error attaching DEPTH_COMPONENT16');

    // Test STENCIL_INDEX8
    const rbStencil = gl.createRenderbuffer();
    gl.bindRenderbuffer(gl.RENDERBUFFER, rbStencil);
    gl.renderbufferStorage(gl.RENDERBUFFER, gl.STENCIL_INDEX8, 2, 2);
    gl.framebufferRenderbuffer(gl.FRAMEBUFFER, gl.STENCIL_ATTACHMENT, gl.RENDERBUFFER, rbStencil);

    assert.strictEqual(gl.getError(), gl.NO_ERROR, 'Should be no error attaching STENCIL_INDEX8');

    // Test DEPTH_STENCIL
    const rbDepthStencil = gl.createRenderbuffer();
    gl.bindRenderbuffer(gl.RENDERBUFFER, rbDepthStencil);
    gl.renderbufferStorage(gl.RENDERBUFFER, gl.DEPTH_STENCIL, 2, 2);
    gl.framebufferRenderbuffer(gl.FRAMEBUFFER, gl.DEPTH_STENCIL_ATTACHMENT, gl.RENDERBUFFER, rbDepthStencil);

    assert.strictEqual(gl.getError(), gl.NO_ERROR, 'Should be no error attaching DEPTH_STENCIL');

    // Cleanup
    gl.deleteRenderbuffer(rbDepth);
    gl.deleteRenderbuffer(rbStencil);
    gl.deleteRenderbuffer(rbDepthStencil);
    gl.deleteFramebuffer(fb);
    gl.destroy();
});
