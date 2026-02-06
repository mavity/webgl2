import { test } from 'node:test';
import assert from 'node:assert';
import { webGL2 } from '../../index.js';
import { PACK_FLOAT_GLSL, unpackFloat, canonicalize } from './-core.js';

function det2(m) {
  // m is column-major [a00,a10,a01,a11]
  return m[0] * m[3] - m[2] * m[1];
}

function det3(m) {
  // column-major: a00=0 a10=1 a20=2 a01=3 a11=4 a21=5 a02=6 a12=7 a22=8
  const a00 = m[0], a01 = m[3], a02 = m[6];
  const a10 = m[1], a11 = m[4], a12 = m[7];
  const a20 = m[2], a21 = m[5], a22 = m[8];
  return a00 * (a11 * a22 - a12 * a21) - a01 * (a10 * a22 - a12 * a20) + a02 * (a10 * a21 - a11 * a20);
}

test('Math Builtin: determinant', async (t) => {
  const gl = await webGL2({ size: { width: 1, height: 1 } });
  const vs = `#version 300 es
        in vec4 position;
        void main() { gl_Position = position; }
    `;

  const fs2 = `#version 300 es
        precision highp float;
        uniform vec2 u_c0;
        uniform vec2 u_c1;
        out vec4 outColor;
        void main() {
            mat2 u_m = mat2(u_c0, u_c1);
            float d = determinant(u_m);
            ${PACK_FLOAT_GLSL('d')}
        }
    `;

  const fs3 = `#version 300 es
        precision highp float;
        uniform vec3 u_c0;
        uniform vec3 u_c1;
        uniform vec3 u_c2;
        out vec4 outColor;
        void main() {
            mat3 u_m = mat3(u_c0, u_c1, u_c2);
            float d = determinant(u_m);
            ${PACK_FLOAT_GLSL('d')}
        }
    `;

  // mat2 cases
  const program2 = gl.createProgram();
  const vShader2 = gl.createShader(gl.VERTEX_SHADER);
  gl.shaderSource(vShader2, vs);
  gl.compileShader(vShader2);
  const fShader2 = gl.createShader(gl.FRAGMENT_SHADER);
  gl.shaderSource(fShader2, fs2);
  gl.compileShader(fShader2);
  gl.attachShader(program2, vShader2);
  gl.attachShader(program2, fShader2);
  gl.linkProgram(program2);
  if (!gl.getProgramParameter(program2, gl.LINK_STATUS)) throw new Error(gl.getProgramInfoLog(program2));
  gl.useProgram(program2);
  const uC0_2 = gl.getUniformLocation(program2, 'u_c0');
  const uC1_2 = gl.getUniformLocation(program2, 'u_c1');

  // manual determinant shader to cross-check
  const fs2_manual = `#version 300 es
        precision highp float;
        uniform vec2 u_c0;
        uniform vec2 u_c1;
        out vec4 outColor;
        void main() {
            float d = u_c0.x * u_c1.y - u_c1.x * u_c0.y;
            ${PACK_FLOAT_GLSL('d')}
        }
    `;
  const program2_manual = gl.createProgram();
  const vShader2_manual = gl.createShader(gl.VERTEX_SHADER);
  gl.shaderSource(vShader2_manual, vs);
  gl.compileShader(vShader2_manual);
  const fShader2_manual = gl.createShader(gl.FRAGMENT_SHADER);
  gl.shaderSource(fShader2_manual, fs2_manual);
  gl.compileShader(fShader2_manual);
  gl.attachShader(program2_manual, vShader2_manual);
  gl.attachShader(program2_manual, fShader2_manual);
  gl.linkProgram(program2_manual);
  if (!gl.getProgramParameter(program2_manual, gl.LINK_STATUS)) throw new Error(gl.getProgramInfoLog(program2_manual));
  gl.useProgram(program2_manual);
  const uC0_2_m = gl.getUniformLocation(program2_manual, 'u_c0');
  const uC1_2_m = gl.getUniformLocation(program2_manual, 'u_c1');


  const m2cases = [
    [1, 0, 0, 1], // identity => det 1
    [2, 3, 4, 5],
    [0, 1, 2, 3],
  ];

  for (const m of m2cases) {
    // m is column-major [a00,a10,a01,a11]
    gl.uniform2f(uC0_2, m[0], m[1]);
    gl.uniform2f(uC1_2, m[2], m[3]);
    gl.drawArrays(gl.POINTS, 0, 1);
    const pixels = new Uint8Array(4);
    gl.readPixels(0, 0, 1, 1, gl.RGBA, gl.UNSIGNED_BYTE, pixels);
    const result = unpackFloat(pixels);

    // also check manual shader for cross-check
    gl.useProgram(program2_manual);
    gl.uniform2f(uC0_2_m, m[0], m[1]);
    gl.uniform2f(uC1_2_m, m[2], m[3]);
    gl.drawArrays(gl.POINTS, 0, 1);
    const pixels2 = new Uint8Array(4);
    gl.readPixels(0, 0, 1, 1, gl.RGBA, gl.UNSIGNED_BYTE, pixels2);
    const manual = unpackFloat(pixels2);

    const expected = det2(m);
    console.log('mat2', m, 'builtin', result, 'manual', manual, 'expected', expected);
    assert.equal(canonicalize(manual), canonicalize(expected), `manual determinant mat2 failed for ${m}`);
    assert.equal(canonicalize(result), canonicalize(expected), `determinant mat2 failed for ${m}`);
  }

  // mat3 cases
  const program3 = gl.createProgram();
  const vShader3 = gl.createShader(gl.VERTEX_SHADER);
  gl.shaderSource(vShader3, vs);
  gl.compileShader(vShader3);
  const fShader3 = gl.createShader(gl.FRAGMENT_SHADER);
  gl.shaderSource(fShader3, fs3);
  gl.compileShader(fShader3);
  gl.attachShader(program3, vShader3);
  gl.attachShader(program3, fShader3);
  gl.linkProgram(program3);
  if (!gl.getProgramParameter(program3, gl.LINK_STATUS)) throw new Error(gl.getProgramInfoLog(program3));
  gl.useProgram(program3);
  const uC0_3 = gl.getUniformLocation(program3, 'u_c0');
  const uC1_3 = gl.getUniformLocation(program3, 'u_c1');
  const uC2_3 = gl.getUniformLocation(program3, 'u_c2');

  const fs3_manual = `#version 300 es
        precision highp float;
        uniform vec3 u_c0;
        uniform vec3 u_c1;
        uniform vec3 u_c2;
        out vec4 outColor;
        void main() {
            mat3 m = mat3(u_c0, u_c1, u_c2);
            float d = m[0][0]*(m[1][1]*m[2][2] - m[1][2]*m[2][1]) - m[0][1]*(m[1][0]*m[2][2] - m[1][2]*m[2][0]) + m[0][2]*(m[1][0]*m[2][1] - m[1][1]*m[2][0]);
            ${PACK_FLOAT_GLSL('d')}
        }
    `;
  const program3_manual = gl.createProgram();
  const vShader3_manual = gl.createShader(gl.VERTEX_SHADER);
  gl.shaderSource(vShader3_manual, vs);
  gl.compileShader(vShader3_manual);
  const fShader3_manual = gl.createShader(gl.FRAGMENT_SHADER);
  gl.shaderSource(fShader3_manual, fs3_manual);
  gl.compileShader(fShader3_manual);
  gl.attachShader(program3_manual, vShader3_manual);
  gl.attachShader(program3_manual, fShader3_manual);
  gl.linkProgram(program3_manual);
  if (!gl.getProgramParameter(program3_manual, gl.LINK_STATUS)) throw new Error(gl.getProgramInfoLog(program3_manual));
  gl.useProgram(program3_manual);
  const uC0_3_m = gl.getUniformLocation(program3_manual, 'u_c0');
  const uC1_3_m = gl.getUniformLocation(program3_manual, 'u_c1');
  const uC2_3_m = gl.getUniformLocation(program3_manual, 'u_c2');

  const m3cases = [
    [1, 0, 0, 0, 1, 0, 0, 0, 1],
    [2, 3, 5, 7, 11, 13, 17, 19, 23],
    [0, 1, 2, 3, 4, 5, 6, 7, 8],
  ];

  for (const m of m3cases) {
    const fs_case = `#version 300 es
        precision highp float;
        out vec4 outColor;
        void main() {
            mat3 u_m = mat3(${m.join(',')});
            float d = determinant(u_m);
            ${PACK_FLOAT_GLSL('d')}
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

    const expected = det3(m);
    assert.equal(canonicalize(result), canonicalize(expected), `determinant mat3 failed for ${m}`);
  }
});