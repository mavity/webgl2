import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('resumeTransformFeedback resumes paused transform feedback', async () => {
  const gl = await webGL2();
  try {
    const tf = gl.createTransformFeedback();
    gl.bindTransformFeedback(gl.TRANSFORM_FEEDBACK, tf);
    gl.beginTransformFeedback(gl.TRIANGLES);
    gl.pauseTransformFeedback();
    assert.doesNotThrow(() => gl.resumeTransformFeedback());
  } finally { gl.destroy(); }
});
