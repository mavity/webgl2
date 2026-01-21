import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('Shared function table exists', async () => {
  const gl = await webGL2();
  assert.ok(gl._sharedTable, 'Context should have _sharedTable');
  assert.ok(gl._tableAllocator, 'Context should have _tableAllocator');
  assert.strictEqual(gl._sharedTable.length, 8192, 'Table should have initial size 8192');
  gl.destroy();
});

test('Table allocator works', async () => {
  const gl = await webGL2();
  const idx1 = gl._tableAllocator.allocate();
  const idx2 = gl._tableAllocator.allocate();
  assert.notStrictEqual(idx1, idx2, 'Should allocate different indices');
  
  gl._tableAllocator.free(idx1);
  const idx3 = gl._tableAllocator.allocate();
  assert.strictEqual(idx3, idx1, 'Should reuse freed index');
  
  gl.destroy();
});
