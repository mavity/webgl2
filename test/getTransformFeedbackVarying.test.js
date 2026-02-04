import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('getTransformFeedbackVarying returns varying info', async () => {
  const gl = await webGL2();
  try {
    const p = gl.createProgram();
    gl.transformFeedbackVaryings(p, ['a'], gl.INTERLEAVED_ATTRIBS);
    const v = gl.getTransformFeedbackVarying(p, 0);
    assert.strictEqual(v.name, 'a');
    assert.strictEqual(v.size, 1);
  } finally { gl.destroy(); }
});
