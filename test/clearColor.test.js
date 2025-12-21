import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('clearColor sets and gets state', async () => {
  const gl = await webGL2();
  try { 
    gl.clearColor(0.1, 0.2, 0.3, 0.4); 
    const color = gl.getParameter(gl.COLOR_CLEAR_VALUE);
    // Use a small epsilon for float comparison if needed, but here they should be exact
    assert.strictEqual(Math.abs(color[0] - 0.1) < 0.0001, true);
    assert.strictEqual(Math.abs(color[1] - 0.2) < 0.0001, true);
    assert.strictEqual(Math.abs(color[2] - 0.3) < 0.0001, true);
    assert.strictEqual(Math.abs(color[3] - 0.4) < 0.0001, true);
  } finally { gl.destroy(); }
});
