import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('createTransformFeedback returns a transform feedback object', async () => {
  const gl = await webGL2();
  try {
    const tf = gl.createTransformFeedback();
    assert.ok(tf && typeof tf === 'object');
    assert.ok(gl.isTransformFeedback(tf));
  } finally { gl.destroy(); }
});
