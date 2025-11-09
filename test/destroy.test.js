import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('destroy does not throw', async () => {
  const gl = await webGL2();
  try {
    // should not throw
    gl.destroy();
  } finally {
    // ensure cleanup is idempotent
    try { gl.destroy(); } catch (e) { /* ignore */ }
  }
});

test('destroy marks context as destroyed', async () => {
  const gl = await webGL2();
  try {
    gl.destroy();
    // subsequent API calls should indicate destroyed state
    assert.throws(() => gl.createTexture(), /context has been destroyed/);
  } finally {
    try { gl.destroy(); } catch (e) { /* ignore */ }
  }
});

test('multiple destroys are idempotent', async () => {
  const gl = await webGL2();
  try {
    gl.destroy();
    // second destroy should be a no-op and not throw
    gl.destroy();
  } finally {
    try { gl.destroy(); } catch (e) { /* ignore */ }
  }
});
