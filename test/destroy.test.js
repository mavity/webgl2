import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2, WasmWebGL2RenderingContext } from '../index.js';

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

test('resources from destroyed context are invalid', async () => {
  const gl = await webGL2();
  const tex = gl.createTexture();
  gl.destroy();
  
  // Even if we have the texture object, using it with the context should fail
  // because the context itself is marked as destroyed.
  assert.throws(() => gl.bindTexture(gl.TEXTURE_2D, tex), /context has been destroyed/);
});

test('can create new context after destroying old one', async () => {
  const gl1 = await webGL2();
  const handle1 = gl1._ctxHandle;
  gl1.destroy();
  
  const gl2 = await webGL2();
  const handle2 = gl2._ctxHandle;
  
  assert.notStrictEqual(gl1, gl2);
  // Handles might be reused or incremented, but the objects must be distinct
  assert.ok(gl2 instanceof WasmWebGL2RenderingContext);
  
  // gl2 should work
  const tex = gl2.createTexture();
  assert.ok(tex);
  
  gl2.destroy();
});

test('destroying one context does not affect others', async () => {
  const gl1 = await webGL2();
  const gl2 = await webGL2();
  
  const tex1 = gl1.createTexture();
  const tex2 = gl2.createTexture();
  
  gl1.destroy();
  
  // gl1 should be destroyed
  assert.throws(() => gl1.bindTexture(gl1.TEXTURE_2D, tex1), /context has been destroyed/);
  
  // gl2 should still work
  gl2.bindTexture(gl2.TEXTURE_2D, tex2);
  gl2.destroy();
});

test('cannot use resource from one context in another', async () => {
  const gl1 = await webGL2();
  const gl2 = await webGL2();
  
  const tex1 = gl1.createTexture();
  
  // Attempting to use tex1 (from gl1) in gl2 should fail
  // Note: tex1 is a WebGLTexture wrapper that just holds a handle (u32)
  assert.throws(() => gl2.bindTexture(gl2.TEXTURE_2D, tex1), /WASM error 1: texture not found/);
  
  gl1.destroy();
  gl2.destroy();
});
