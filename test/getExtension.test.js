import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('getExtension returns supported extensions', async () => {
  const gl = await webGL2();
  try {
    assert.notEqual(gl.getExtension('EXT_color_buffer_float'), null);
    assert.equal(gl.getExtension('NON_EXISTENT'), null);
  } finally { gl.destroy(); }
});
