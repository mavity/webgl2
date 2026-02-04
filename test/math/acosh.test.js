
import { test } from 'node:test';
import assert from 'node:assert';
import { webGL2 } from '../../index.js';
import { PACK_FLOAT_GLSL, unpackFloat, canonicalize } from './-core.js';

test('Math Builtin: acosh', async (t) => {
    const gl = await webGL2({ size: { width: 1, height: 1 } });
    
    const vs = `#version 300 es
        in vec4 position;
        void main() {
            gl_Position = position;
        }
    `;

    const fs = `#version 300 es
        precision highp float;
        uniform float u_input;
        out vec4 outColor;
        void main() {
            ${PACK_FLOAT_GLSL('acosh(u_input)')}
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
    const uInputLoc = gl.getUniformLocation(program, 'u_input');

    const testValues = [1.0, 2.0, 5.0, 10.0];

    for (const val of testValues) {
        gl.uniform1f(uInputLoc, val);
        gl.drawArrays(gl.POINTS, 0, 1);

        const pixels = new Uint8Array(4);
        gl.readPixels(0, 0, 1, 1, gl.RGBA, gl.UNSIGNED_BYTE, pixels);
        
        const result = unpackFloat(pixels);
        const expected = Math.acosh(val);

        assert.equal(canonicalize(result), canonicalize(expected), `acosh(${val}) failed`);
    }
});
