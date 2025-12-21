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
  } finally {
    gl.destroy();
  }
});
