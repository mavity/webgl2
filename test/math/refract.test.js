import { test } from 'node:test';
import assert from 'node:assert';
import { webGL2 } from '../../index.js';
import { PACK_FLOAT_GLSL, unpackFloat, canonicalize } from './-core.js';

function computeRefract(I, N, eta) {
    const dot = I[0]*N[0] + I[1]*N[1] + I[2]*N[2];
    const k = 1.0 - eta * eta * (1.0 - dot * dot);
    if (k < 0.0) return [0.0, 0.0, 0.0];
    const s = eta;
    const t = (eta * dot + Math.sqrt(k));
    return [
        s*I[0] - t*N[0],
        s*I[1] - t*N[1],
        s*I[2] - t*N[2],
    ];
}

test('Math Builtin: refract', async (t) => {
    const gl = await webGL2({ size: { width: 1, height: 1 } });
    const vs = `#version 300 es
        in vec4 position;
        void main() { gl_Position = position; }
    `;
    const fs = `#version 300 es
        precision highp float;
        uniform vec3 u_i;
        uniform vec3 u_n;
        uniform float u_eta;
        out vec4 outColor;
        void main() {
            vec3 r = refract(u_i, u_n, u_eta);
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
    const uILoc = gl.getUniformLocation(program, 'u_i');
    const uNLoc = gl.getUniformLocation(program, 'u_n');
    const uEtaLoc = gl.getUniformLocation(program, 'u_eta');

    const cases = [
        { I: [0, 0, -1], N: [0, 0, 1], eta: 1.0 },        // normal incidence
        { I: [0, 0.5, -0.8660254], N: [0, 0, 1], eta: 0.66 }, // oblique, possible TIR
        { I: [0.1, -0.2, -0.97], N: [0, 0, 1], eta: 1.5 },
    ];

    for (const c of cases) {
        gl.uniform3f(uILoc, c.I[0], c.I[1], c.I[2]);
        gl.uniform3f(uNLoc, c.N[0], c.N[1], c.N[2]);
        gl.uniform1f(uEtaLoc, c.eta);
        gl.drawArrays(gl.POINTS, 0, 1);
        const pixels = new Uint8Array(4);
        gl.readPixels(0, 0, 1, 1, gl.RGBA, gl.UNSIGNED_BYTE, pixels);
        const result = unpackFloat(pixels);
        const expected = computeRefract(c.I, c.N, c.eta)[0];
        assert.equal(canonicalize(result), canonicalize(expected), `refract(${JSON.stringify(c)}) x failed`);
    }
});