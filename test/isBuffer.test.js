import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('isBuffer works', async () => {
  const gl = await webGL2();
  try {
    const b = gl.createBuffer();
    assert.equal(gl.isBuffer(b), true, 'new buffer is valid');

    gl.bindBuffer(gl.ARRAY_BUFFER, b);
    assert.equal(gl.isBuffer(b), true, 'bound buffer is valid');

    gl.deleteBuffer(b);
    assert.equal(gl.isBuffer(b), false, 'deleted buffer is invalid');

    assert.equal(gl.isBuffer(null), false, 'null is not a buffer');
    assert.equal(gl.isBuffer(undefined), false, 'undefined is not a buffer');
    assert.equal(gl.isBuffer({}), false, 'object is not a buffer');
    assert.equal(gl.isBuffer(123), false, 'number is not a buffer');
  } finally {
    gl.destroy();
  }
});

test('isBuffer context isolation', async () => {
  const gl1 = await webGL2();
  const gl2 = await webGL2();
  try {
    const b1 = gl1.createBuffer();
    assert.equal(gl1.isBuffer(b1), true);
    assert.equal(gl2.isBuffer(b1), false);
  } finally {
    gl1.destroy();
    gl2.destroy();
  }
});
