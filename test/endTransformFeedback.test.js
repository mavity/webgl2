import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('endTransformFeedback stops transform feedback after begin', async () => {
  const gl = await webGL2();
  try {
    const tf = gl.createTransformFeedback();
    gl.bindTransformFeedback(gl.TRANSFORM_FEEDBACK, tf);
    gl.beginTransformFeedback(gl.TRIANGLES);
    assert.doesNotThrow(() => gl.endTransformFeedback());
  } finally { gl.destroy(); }
});
