import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('deleteRenderbuffer works', async () => {
  const gl = await webGL2();
  const rb = gl.createRenderbuffer();
  gl.deleteRenderbuffer(rb);
  assert.equal(gl.getError(), gl.NO_ERROR);
  gl.destroy();
});
