import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('getSupportedExtensions returns list of extensions', async () => {
  const gl = await webGL2();
  try {
    const extensions = gl.getSupportedExtensions();
    assert.ok(Array.isArray(extensions));
    assert.ok(extensions.includes('EXT_color_buffer_float'));
  } finally { gl.destroy(); }
});
