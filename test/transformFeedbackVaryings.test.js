import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('transformFeedbackVaryings stores varyings on program', async () => {
  const gl = await webGL2();
  try {
    const p = gl.createProgram();
    gl.transformFeedbackVaryings(p, ['a','b'], gl.INTERLEAVED_ATTRIBS);
    // No exception, and getTransformFeedbackVarying should return the names
    const v0 = gl.getTransformFeedbackVarying(p, 0);
    assert.strictEqual(v0.name, 'a');
  } finally { gl.destroy(); }
});
