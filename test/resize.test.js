
import { webGL2 } from '../index.js';
import assert from 'assert';

async function testResize() {
    console.log('Testing resize functionality...');

    // Test 1: Default size
    const gl1 = await webGL2();
    assert.strictEqual(gl1.drawingBufferWidth, 640, 'Default width should be 640');
    assert.strictEqual(gl1.drawingBufferHeight, 480, 'Default height should be 480');
    gl1.destroy();
    console.log('✓ Default size correct');

    // Test 2: Custom size in constructor
    const gl2 = await webGL2({ size: { width: 800, height: 600 } });
    assert.strictEqual(gl2.drawingBufferWidth, 800, 'Custom width should be 800');
    assert.strictEqual(gl2.drawingBufferHeight, 600, 'Custom height should be 600');
    gl2.destroy();
    console.log('✓ Custom size in constructor correct');

    // Test 3: Resize method
    const gl3 = await webGL2();
    gl3.resize(1024, 768);
    assert.strictEqual(gl3.drawingBufferWidth, 1024, 'Resized width should be 1024');
    assert.strictEqual(gl3.drawingBufferHeight, 768, 'Resized height should be 768');
    
    // Verify rendering doesn't crash after resize
    // We'll just clear the buffer, which accesses the framebuffer
    gl3.clearColor(1.0, 0.0, 0.0, 1.0);
    gl3.clear(gl3.COLOR_BUFFER_BIT);
    
    // Read pixels from the corner of the new size
    const pixels = new Uint8Array(4);
    gl3.readPixels(1023, 767, 1, 1, gl3.RGBA, gl3.UNSIGNED_BYTE, pixels);
    assert.strictEqual(pixels[0], 255, 'Red channel should be 255');
    assert.strictEqual(pixels[1], 0, 'Green channel should be 0');
    assert.strictEqual(pixels[2], 0, 'Blue channel should be 0');
    assert.strictEqual(pixels[3], 255, 'Alpha channel should be 255');

    gl3.destroy();
    console.log('✓ Resize method and rendering correct');

    // Test 4: Multiple resizes
    const gl4 = await webGL2();
    gl4.resize(100, 100);
    assert.strictEqual(gl4.drawingBufferWidth, 100);
    gl4.resize(200, 200);
    assert.strictEqual(gl4.drawingBufferWidth, 200);
    gl4.destroy();
    console.log('✓ Multiple resizes correct');

    // Test 5: Zero size (should be allowed, though useless)
    const gl5 = await webGL2();
    gl5.resize(0, 0);
    assert.strictEqual(gl5.drawingBufferWidth, 0);
    assert.strictEqual(gl5.drawingBufferHeight, 0);
    gl5.destroy();
    console.log('✓ Zero size correct');
}

testResize().catch(err => {
    console.error('Test failed:', err);
    process.exit(1);
});
