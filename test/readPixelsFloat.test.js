import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('readPixels float formats - RGBA32F', async () => {
  const gl = await webGL2();
  try {
    const tex = gl.createTexture();
    const GL_RGBA32F = 0x8814;
    const GL_RGBA = 0x1908;
    const GL_FLOAT = 0x1406;

    gl.bindTexture(gl.TEXTURE_2D, tex);
    const data = new Float32Array([1.0, 2.0, 3.0, 4.0]);
    gl.texImage2D(gl.TEXTURE_2D, 0, GL_RGBA32F, 1, 1, 0, GL_RGBA, GL_FLOAT, data);

    const fb = gl.createFramebuffer();
    gl.bindFramebuffer(gl.FRAMEBUFFER, fb);
    gl.framebufferTexture2D(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.TEXTURE_2D, tex, 0);

    const out = new Float32Array(4);
    gl.readPixels(0, 0, 1, 1, GL_RGBA, GL_FLOAT, out);

    assert.strictEqual(out[0], 1.0);
    assert.strictEqual(out[1], 2.0);
    assert.strictEqual(out[2], 3.0);
    assert.strictEqual(out[3], 4.0);
  } finally {
    gl.destroy();
  }
});

test('readPixels float formats - R32F', async () => {
  const gl = await webGL2();
  try {
    const tex = gl.createTexture();
    const GL_R32F = 0x822E;
    const GL_RED = 0x1903;
    const GL_FLOAT = 0x1406;

    gl.bindTexture(gl.TEXTURE_2D, tex);
    const data = new Float32Array([123.45]);
    gl.texImage2D(gl.TEXTURE_2D, 0, GL_R32F, 1, 1, 0, GL_RED, GL_FLOAT, data);

    const fb = gl.createFramebuffer();
    gl.bindFramebuffer(gl.FRAMEBUFFER, fb);
    gl.framebufferTexture2D(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.TEXTURE_2D, tex, 0);

    const out = new Float32Array(1);
    gl.readPixels(0, 0, 1, 1, GL_RED, GL_FLOAT, out);

    const expected = new Float32Array([123.45])[0];
    assert.strictEqual(out[0], expected);
  } finally {
    gl.destroy();
  }
});
