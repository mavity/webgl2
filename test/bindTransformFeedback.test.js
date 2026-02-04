import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('bindTransformFeedback binds a transform feedback object', async () => {
  const gl = await webGL2();
  try {
    const tf = gl.createTransformFeedback();
    assert.doesNotThrow(() => gl.bindTransformFeedback(gl.TRANSFORM_FEEDBACK, tf));
  } finally { gl.destroy(); }
});
