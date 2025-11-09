import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('texImage3D throws not implemented', async () => {
  const gl = await webGL2();
  try { assert.throws(() => gl.texImage3D(0,0,0,1,1,1,0,0,0,null), /not implemented/); } finally { gl.destroy(); }
});
