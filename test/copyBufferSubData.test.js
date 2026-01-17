import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('copyBufferSubData copies data between buffers', async () => {
    const gl = await webGL2();
    try {
        const src = gl.createBuffer();
        const dst = gl.createBuffer();
        
        const data = new Uint8Array([1, 2, 3, 4, 5, 6, 7, 8]);
        gl.bindBuffer(gl.ARRAY_BUFFER, src);
        gl.bufferData(gl.ARRAY_BUFFER, data, gl.STATIC_DRAW);
        
        gl.bindBuffer(gl.ARRAY_BUFFER, dst);
        gl.bufferData(gl.ARRAY_BUFFER, 8, gl.STATIC_DRAW);
        
        gl.bindBuffer(gl.COPY_READ_BUFFER, src);
        gl.bindBuffer(gl.COPY_WRITE_BUFFER, dst);
        
        gl.copyBufferSubData(gl.COPY_READ_BUFFER, gl.COPY_WRITE_BUFFER, 0, 0, 8);
        
        assert.equal(gl.getError(), gl.NO_ERROR);
        
    } finally {
        gl.destroy();
    }
});
