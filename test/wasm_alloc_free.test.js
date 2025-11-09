import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('wasm_alloc and wasm_free roundtrip', async () => {
  const gl = await webGL2();
  try {
    const ex = gl._instance.exports;
    if (!ex || typeof ex.wasm_alloc !== 'function' || typeof ex.wasm_free !== 'function') {
      throw new Error('wasm_alloc/wasm_free exports not available');
    }

    const len = 64;
    const ptr = ex.wasm_alloc(len);
    assert(ptr !== 0, 'wasm_alloc returned 0');

    const mem = new Uint8Array(ex.memory.buffer);
    for (let i = 0; i < len; i++) mem[ptr + i] = i & 0xff;

    const code = ex.wasm_free(ptr);
    assert.strictEqual(code, 0, `wasm_free returned ${code}`);
  } finally {
    gl.destroy();
  }
});
