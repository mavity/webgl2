import { test } from 'node:test';
import assert from 'node:assert';
import { webGL2 } from '../../index.js';
import { PACK_FLOAT_GLSL, unpackFloat, canonicalize } from './-core.js';

// We'll check A * inverse(A) â‰ˆ I by reading element (0,0) which should be 1.0

test('Math Builtin: inverse mat2', async (t) => {
  const gl = await webGL2({ size: { width: 1, height: 1 } });
  const vs = `#version 300 es
        in vec4 position;
        void main() { gl_Position = position; }
    `;
  const fs = `#version 300 es
        precision highp float;
        uniform vec2 u_c0;
        uniform vec2 u_c1;
        out vec4 outColor;
        void main() {
            mat2 u_m = mat2(u_c0, u_c1);
            mat2 invm = inverse(u_m);
            mat2 prod = u_m * invm;
            ${PACK_FLOAT_GLSL('prod[0][0]')}
        }
    `;

  const cases = [
    [1, 0, 0, 1], // identity
    [2, 0, 0, 3],
    [4, 7, 2, 5],
  ];

  for (const m of cases) {
    const fs_case = `#version 300 es
        precision highp float;
        out vec4 outColor;
        void main() {
            mat2 u_m = mat2(${m.join(',')});
            mat2 invm = inverse(u_m);
            mat2 prod = u_m * invm;
            ${PACK_FLOAT_GLSL('prod[0][0]')}
        }
    `;
    const program_case = gl.createProgram();
    const vShader_case = gl.createShader(gl.VERTEX_SHADER);
    gl.shaderSource(vShader_case, vs);
    gl.compileShader(vShader_case);
    const fShader_case = gl.createShader(gl.FRAGMENT_SHADER);
    gl.shaderSource(fShader_case, fs_case);
    gl.compileShader(fShader_case);
    gl.attachShader(program_case, vShader_case);
    gl.attachShader(program_case, fShader_case);
    gl.linkProgram(program_case);
    if (!gl.getProgramParameter(program_case, gl.LINK_STATUS)) throw new Error(gl.getProgramInfoLog(program_case));
    gl.useProgram(program_case);
    gl.drawArrays(gl.POINTS, 0, 1);
    const pixels = new Uint8Array(4);
    gl.readPixels(0, 0, 1, 1, gl.RGBA, gl.UNSIGNED_BYTE, pixels);
    const result = unpackFloat(pixels);
    assert.equal(canonicalize(result), canonicalize(1.0), `inverse(mat2) failed for ${m}`);
  }
});


test('Math Builtin: inverse mat3', async (t) => {
  const gl = await webGL2({ size: { width: 1, height: 1 } });
  const vs = `#version 300 es
        in vec4 position;
        void main() { gl_Position = position; }
    `;
  const fs = `#version 300 es
        precision highp float;
        uniform vec3 u_c0;
        uniform vec3 u_c1;
        uniform vec3 u_c2;
        out vec4 outColor;
        void main() {
            mat3 u_m = mat3(u_c0, u_c1, u_c2);
            mat3 invm = inverse(u_m);
            mat3 prod = u_m * invm;
            ${PACK_FLOAT_GLSL('prod[0][0]')}
        }
    `;

  const cases = [
    [1, 0, 0, 0, 1, 0, 0, 0, 1],
    [2, 3, 5, 7, 11, 13, 17, 19, 23],
    [2, 0, 1, 0, 3, 4, 5, 6, 7],
  ];

  for (const m of cases) {
    const fs_case = `#version 300 es
        precision highp float;
        out vec4 outColor;
        void main() {
            mat3 u_m = mat3(${m.join(',')});
            mat3 invm = inverse(u_m);
            mat3 prod = u_m * invm;
            ${PACK_FLOAT_GLSL('prod[0][0]')}
        }
    `;
    const program_case = gl.createProgram();
    const vShader_case = gl.createShader(gl.VERTEX_SHADER);
    gl.shaderSource(vShader_case, vs);
    gl.compileShader(vShader_case);
    const fShader_case = gl.createShader(gl.FRAGMENT_SHADER);
    gl.shaderSource(fShader_case, fs_case);
    gl.compileShader(fShader_case);
    gl.attachShader(program_case, vShader_case);
    gl.attachShader(program_case, fShader_case);
    gl.linkProgram(program_case);
    if (!gl.getProgramParameter(program_case, gl.LINK_STATUS)) throw new Error(gl.getProgramInfoLog(program_case));
    gl.useProgram(program_case);
    gl.drawArrays(gl.POINTS, 0, 1);
    const pixels = new Uint8Array(4);
    gl.readPixels(0, 0, 1, 1, gl.RGBA, gl.UNSIGNED_BYTE, pixels);
    const result = unpackFloat(pixels);
    // target is approx 1.0
    assert.equal(canonicalize(result), canonicalize(1.0), `inverse(mat3) failed for ${m}`);
  }
});
