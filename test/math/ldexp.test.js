import { test } from 'node:test';
import assert from 'node:assert';
import { webGL2 } from '../../index.js';
import { PACK_FLOAT_GLSL, unpackFloat, canonicalize } from './-core.js';

test('Math Builtin: ldexp', async (t) => {
    const gl = await webGL2({ size: { width: 1, height: 1 } });

    const vs = `#version 300 es
        in vec4 position;
        void main() { gl_Position = position; }
    `;

    const fs = `#version 300 es
        precision highp float;
        uniform float u_mant;
        uniform int u_exp;
        out vec4 outColor;
        void main() {
            ${PACK_FLOAT_GLSL('ldexp(u_mant, u_exp)')}
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
    const uMantLoc = gl.getUniformLocation(program, 'u_mant');
    const uExpLoc = gl.getUniformLocation(program, 'u_exp');

    const mantissas = [1.0, 2.5, -1.5, 0.5];
    const exps = [0, 1, 3, -1];

    for (const m of mantissas) {
        for (const e of exps) {
            gl.uniform1f(uMantLoc, m);
            gl.uniform1i(uExpLoc, e);
            gl.drawArrays(gl.POINTS, 0, 1);
            const pixels = new Uint8Array(4);
            gl.readPixels(0, 0, 1, 1, gl.RGBA, gl.UNSIGNED_BYTE, pixels);
            const result = unpackFloat(pixels);
            const expected = m * Math.pow(2, e);
            assert.equal(canonicalize(result), canonicalize(expected), `ldexp(${m}, ${e}) failed`);
        }
    }
});


test('Math Builtin: ldexp vector', async (t) => {
    const gl = await webGL2({ size: { width: 1, height: 1 } });

    const vs = `#version 300 es
        in vec4 position;
        void main() { gl_Position = position; }
    `;

    const fs = `#version 300 es
        precision highp float;
        uniform vec2 u_mant;
        uniform int u_exp_x;
        uniform int u_exp_y;
        out vec4 outColor;
        void main() {
            vec2 r = ldexp(u_mant, ivec2(u_exp_x, u_exp_y));
            ${PACK_FLOAT_GLSL('r.x')}
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
    gl.uniform2f(gl.getUniformLocation(program, 'u_mant'), 1.5, -2.0);
    gl.uniform1i(gl.getUniformLocation(program, 'u_exp_x'), 2);
    gl.uniform1i(gl.getUniformLocation(program, 'u_exp_y'), -1);

    gl.drawArrays(gl.POINTS, 0, 1);
    const pixels = new Uint8Array(4);
    gl.readPixels(0, 0, 1, 1, gl.RGBA, gl.UNSIGNED_BYTE, pixels);
    const result = unpackFloat(pixels);
    const expected = 1.5 * Math.pow(2, 2);
    assert.equal(canonicalize(result), canonicalize(expected), `ldexp(vec2).x failed`);
});
