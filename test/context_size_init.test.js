
import { webGL2 } from '../index.js';
import { test } from 'node:test';
import assert from 'node:assert';

test('Context Creation Size', async (t) => {
    await t.test('Default size should be 640x480', async () => {
        const gl = await webGL2();
        assert.strictEqual(gl.drawingBufferWidth, 640);
        assert.strictEqual(gl.drawingBufferHeight, 480);
        gl.destroy();
    });

    await t.test('Explicit size should be respected', async () => {
        const gl = await webGL2({ size: { width: 800, height: 600 } });
        assert.strictEqual(gl.drawingBufferWidth, 800);
        assert.strictEqual(gl.drawingBufferHeight, 600);
        gl.destroy();
    });

    await t.test('Partial size (width only) should use default height', async () => {
        const gl = await webGL2({ size: { width: 100 } });
        assert.strictEqual(gl.drawingBufferWidth, 100);
        assert.strictEqual(gl.drawingBufferHeight, 480);
        gl.destroy();
    });

    await t.test('Partial size (height only) should use default width', async () => {
        const gl = await webGL2({ size: { height: 200 } });
        assert.strictEqual(gl.drawingBufferWidth, 640);
        assert.strictEqual(gl.drawingBufferHeight, 200);
        gl.destroy();
    });

    await t.test('Resize should update dimensions', async () => {
        const gl = await webGL2();
        gl.resize(300, 300);
        assert.strictEqual(gl.drawingBufferWidth, 300);
        assert.strictEqual(gl.drawingBufferHeight, 300);
        gl.destroy();
    });
});
