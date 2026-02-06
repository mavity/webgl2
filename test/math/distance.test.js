import { test } from 'node:test';
import assert from 'node:assert';
import { webGL2 } from '../../index.js';
import { PACK_FLOAT_GLSL, unpackFloat, canonicalize } from './-core.js';

function computeDistance(a, b) {
  let s = 0.0;
  for (let i = 0; i < a.length; ++i) {
    const d = a[i] - b[i];
    s += d * d;
  }
  return Math.sqrt(s);
}

test('Math Builtin: distance', async (t) => {
  const gl = await webGL2({ size: { width: 1, height: 1 } });
  const vs = `#version 300 es
        in vec4 position;
        void main() { gl_Position = position; }
    `;
  const fs = `#version 300 es
        precision highp float;
        uniform vec3 u_a;
        uniform vec3 u_b;
        out vec4 outColor;
        void main() {
            float d = distance(u_a, u_b);
            ${PACK_FLOAT_GLSL('d')}
        }
    `;

  // We'll compile a shader per-case using literal constants to avoid uniform path issues
  const cases = [
    { a: [0, 0, 0], b: [1, 0, 0] },
    { a: [0.1, -0.2, 0.3], b: [0.5, 0.5, -0.1] },
    { a: [1, 2, 3], b: [4, 5, 6] },
  ];

  for (const c of cases) {
    const fs_case = `#version 300 es
        precision highp float;
        out vec4 outColor;
        void main() {
            float d = distance(vec3(${c.a.join(',')}), vec3(${c.b.join(',')}));
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
    if (!gl.getProgramParameter(program_case, gl.LINK_STATUS)) {
      throw new Error(gl.getProgramInfoLog(program_case));
    }

    gl.useProgram(program_case);
    gl.drawArrays(gl.POINTS, 0, 1);
    const pixels = new Uint8Array(4);
    gl.readPixels(0, 0, 1, 1, gl.RGBA, gl.UNSIGNED_BYTE, pixels);
    const result = unpackFloat(pixels);
    const expected = computeDistance(c.a, c.b);
    assert.equal(canonicalize(result), canonicalize(expected), `distance(${JSON.stringify(c)}) failed`);
  }
});