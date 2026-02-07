// @ts-check

import assert from 'node:assert';
import { test } from 'node:test';


import { /** getGL, resetGL, */ createTestTexture } from '../../external/mavity/gravity/test-utils.js';
import { GraphLayout } from '../../external/mavity/graph/layout.js';

import { webGL2 } from '../../index.js';

// Polyfill for browser globals in Node tests
if (typeof global.window === 'undefined') {
  // Expose a minimal window object for tests that reference window
  global.window = global;
}

test('GraphLayout: constructor validates missing textures', async () => {
  const gl = await getGL();
  
  // @ts-ignore
  assert.throws(() => new GraphLayout({
    gl,
    particleCount: 128,
    edgeCount: 128
  }), /Missing required textures/);
  
  resetGL();
});

test('GraphLayout: basic allocation and single-frame run', async () => {
  const gl = await getGL();
  
  const particleCount = 10;
  const edgeCount = 20;

  // Mock textures
  const texPos = createTestTexture(gl, 4, 4, new Float32Array(16 * 4));
  const texVel = createTestTexture(gl, 4, 4, new Float32Array(16 * 4));
  const texMeta = createTestTexture(gl, 4, 4, new Float32Array(16 * 4));
  const texPtr = createTestTexture(gl, 4, 4, new Float32Array(16 * 4), 'R32F');
  const texStore = createTestTexture(gl, 8, 4, new Float32Array(32 * 4), 'R32F');

  const layout = new GraphLayout({
    gl,
    particleCount,
    edgeCount,
    texPosition: texPos,
    texVelocity: texVel,
    texIdMassTint: texMeta,
    texEdgePtr: texPtr,
    texEdgeStore: texStore
  });

  assert.strictEqual(layout.particleCount, particleCount);
  assert.strictEqual(layout.edgeCount, edgeCount);
  assert.strictEqual(layout.renderCount, 0);

  // Run one frame
  layout.run();

  assert.strictEqual(layout.renderCount, 1);
  assert.strictEqual(layout.frameCounter, 1);
  assert.strictEqual(layout.passCounter, 1);

  // Verify reflection
  const snapshot = layout.valueOf({ pixels: false });
  assert.strictEqual(snapshot.renderCount, 1);
  assert.ok(snapshot.position, 'Position snapshot should exist');
  assert.ok(snapshot.toString().includes('GraphLayout'), 'toString should contain class name');

  layout.dispose();
  // Textures are removed by layout.dispose(); avoid double-delete
  resetGL();
});

async function getGL() {
  const gl = await webGL2();
  return gl;
}

function resetGL() {
}