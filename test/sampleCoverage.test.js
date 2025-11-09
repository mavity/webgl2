import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('sampleCoverage throws not implemented', async () => {
  const gl = await webGL2();
  try { assert.throws(() => gl.sampleCoverage(1.0, false), /not implemented/); } finally { gl.destroy(); }
});
