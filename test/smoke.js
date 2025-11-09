#!/usr/bin/env node

/**
 * Smoke Test: index2.js WebGL2 Context
 *
 * Tests the full pipeline:
 * 1. Load and instantiate WASM
 * 2. Create a context
 * 3. Create a texture
 * 4. Upload a 1x1 pixel (CornflowerBlue: 100, 149, 237, 255)
 * 5. Create a framebuffer
 * 6. Attach texture to framebuffer
 * 7. Read pixels back
 * 8. Verify pixel color matches
 * 9. Destroy context
 */

import { webGL2 } from '../index.js';

async function runSmokeTest() {
  console.log('=== WebGL2 Smoke Test (index2.js) ===\n');

  try {
    console.log('1. Creating WebGL2 context...');
    const gl = await webGL2();
    console.log('   ✓ Context created\n');

    console.log('2. Creating texture...');
    const texHandle = gl.createTexture();
    console.log(`   ✓ Texture created (handle: ${texHandle})\n`);

    console.log('3. Binding texture...');
    gl.bindTexture(0, texHandle);
    console.log('   ✓ Texture bound\n');

    console.log('4. Uploading pixel data (1x1 CornflowerBlue)...');
    // CornflowerBlue: #6495ED = (100, 149, 237) RGBA
    const pixelData = new Uint8Array([100, 149, 237, 255]);
    gl.texImage2D(0, 0, 0, 1, 1, 0, 0, 0, pixelData);
    console.log('   ✓ Pixel data uploaded\n');

    console.log('5. Creating framebuffer...');
    const fbHandle = gl.createFramebuffer();
    console.log(`   ✓ Framebuffer created (handle: ${fbHandle})\n`);

    console.log('6. Binding framebuffer...');
    gl.bindFramebuffer(0, fbHandle);
    console.log('   ✓ Framebuffer bound\n');

    console.log('7. Attaching texture to framebuffer...');
    gl.framebufferTexture2D(0, 0, 0, texHandle, 0);
    console.log('   ✓ Texture attached\n');

    console.log('8. Reading pixels back...');
    const readBuffer = new Uint8Array(4);
    gl.readPixels(0, 0, 1, 1, 0, 0, readBuffer);
    console.log(
      `   ✓ Pixels read: r=${readBuffer[0]}, g=${readBuffer[1]}, b=${readBuffer[2]}, a=${readBuffer[3]}\n`
    );

    console.log('9. Verifying pixel data...');
    const match =
      readBuffer[0] === 100 &&
      readBuffer[1] === 149 &&
      readBuffer[2] === 237 &&
      readBuffer[3] === 255;

    if (match) {
      console.log('   ✓ Pixel data matches expected CornflowerBlue!\n');
    } else {
      console.error('   ✗ Pixel mismatch!');
      console.error(
        `     Expected: [100, 149, 237, 255]\n     Got: [${readBuffer[0]}, ${readBuffer[1]}, ${readBuffer[2]}, ${readBuffer[3]}]`
      );
      process.exit(1);
    }

    console.log('10. Destroying context...');
    gl.destroy();
    console.log('    ✓ Context destroyed\n');

    console.log('=== ✓ Smoke Test PASSED ===\n');
    process.exit(0);
  } catch (e) {
    console.error('\n=== ✗ Smoke Test FAILED ===\n');
    console.error('Error:', e && e.message ? e.message : e);
    if (e.stack) {
      console.error(e.stack);
    }
    process.exit(1);
  }
}

runSmokeTest();
