import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('createRenderbuffer returns a renderbuffer object', async () => {
  const gl = await webGL2();
  const rb = gl.createRenderbuffer();
  assert.ok(rb, 'createRenderbuffer should return an object');
  assert.equal(gl.getError(), gl.NO_ERROR);
  gl.destroy();
});
