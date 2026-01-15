
import { test } from 'node:test';
import assert from 'node:assert';
import { webGL2 } from '../../index.js';
import { PACK_FLOAT_GLSL, unpackFloat, canonicalize } from '../helpers/float-packing.js';

test('Math Builtin: pow', async (t) => {
    const gl = await webGL2({ size: { width: 1, height: 1 } });
    
    const vs = `#version 300 es
        in vec4 position;
        void main() {
            gl_Position = position;
        }
    `;

    const fs = `#version 300 es
        precision highp float;
        uniform float u_x;
        uniform float u_y;
        out vec4 outColor;
        void main() {
            ${PACK_FLOAT_GLSL('pow(u_x, u_y)')}
        }
    `;

    const program = gl.createProgram();
    const vShader = gl.createShader(gl.VERTEX_SHADER);
    gl.shaderSource(vShader, vs);
    gl.compileShader(vShader);
    const fShader = gl.createShader(gl.FRAGMENT_SHADER);
    gl.shaderSource(fShader, fs);
    gl.compileShader(fShader);
    gl.attachShader(program, vShader);
    gl.attachShader(program, fShader);
    gl.linkProgram(program);

    if (!gl.getProgramParameter(program, gl.LINK_STATUS)) {
        throw new Error(gl.getProgramInfoLog(program));
    }

    gl.useProgram(program);
    const uXLoc = gl.getUniformLocation(program, 'u_x');
    const uYLoc = gl.getUniformLocation(program, 'u_y');

    const testPairs = [
        [2.0, 3.0],
        [3.0, 2.0],
        [4.0, 0.5],
        [10.0, -1.0],
        [Math.E, 1.0]
    ];

    for (const [x, y] of testPairs) {
        gl.uniform1f(uXLoc, x);
        gl.uniform1f(uYLoc, y);
        gl.drawArrays(gl.POINTS, 0, 1);

        const pixels = new Uint8Array(4);
        gl.readPixels(0, 0, 1, 1, gl.RGBA, gl.UNSIGNED_BYTE, pixels);
        
        const result = unpackFloat(pixels);
        const expected = Math.pow(x, y);

        assert.equal(canonicalize(result), canonicalize(expected), `pow(${x}, ${y}) failed`);
    }
});
