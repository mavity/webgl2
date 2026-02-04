import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('isEnabled reflects capability state', async () => {
  const gl = await webGL2();
  try {
    assert.strictEqual(gl.isEnabled(gl.SCISSOR_TEST), false);
    gl.enable(gl.SCISSOR_TEST);
    assert.strictEqual(gl.isEnabled(gl.SCISSOR_TEST), true);
    gl.disable(gl.SCISSOR_TEST);
    assert.strictEqual(gl.isEnabled(gl.SCISSOR_TEST), false);

    assert.strictEqual(gl.isEnabled(gl.BLEND), false);
    gl.enable(gl.BLEND);
    assert.strictEqual(gl.isEnabled(gl.BLEND), true);
  } finally {
    gl.destroy();
  }
});
