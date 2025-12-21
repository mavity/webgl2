import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('viewport sets and gets state', async () => {
  const gl = await webGL2();
  try { 
    gl.viewport(10, 20, 30, 40); 
    const vp = gl.getParameter(gl.VIEWPORT);
    assert.deepStrictEqual(Array.from(vp), [10, 20, 30, 40]);
  } finally { gl.destroy(); }
});
