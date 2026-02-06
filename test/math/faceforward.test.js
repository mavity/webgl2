import { test } from 'node:test';
import assert from 'node:assert';
import { webGL2 } from '../../index.js';
import { PACK_FLOAT_GLSL, unpackFloat, canonicalize } from './-core.js';

function computeFaceForward(N, I, Nref) {
  const dot = Nref[0] * I[0] + Nref[1] * I[1] + Nref[2] * I[2];
  if (dot < 0.0) return N;
  return [-N[0], -N[1], -N[2]];
}

test('Math Builtin: faceforward', async (t) => {
  const gl = await webGL2({ size: { width: 1, height: 1 } });
  const vs = `#version 300 es
        in vec4 position;
        void main() { gl_Position = position; }
    `;
  const fs = `#version 300 es
        precision highp float;
        uniform vec3 u_n;
        uniform vec3 u_i;
        uniform vec3 u_nref;
        out vec4 outColor;
        void main() {
            vec3 r = faceforward(u_n, u_i, u_nref);
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
  const uNLoc = gl.getUniformLocation(program, 'u_n');
  const uILoc = gl.getUniformLocation(program, 'u_i');
  const uNRefLoc = gl.getUniformLocation(program, 'u_nref');

  const cases = [
    { N: [0, 0, 1], I: [0, 0, -1], Nref: [0, 0, 1] },
    { N: [0, 1, 0], I: [0, -1, 0], Nref: [0, 0, -1] },
    { N: [0.2, 0.3, 0.9], I: [0.1, 0.2, -0.98], Nref: [0.0, 0.0, 1.0] },
  ];

  for (const c of cases) {
    gl.uniform3f(uNLoc, c.N[0], c.N[1], c.N[2]);
    gl.uniform3f(uILoc, c.I[0], c.I[1], c.I[2]);
    gl.uniform3f(uNRefLoc, c.Nref[0], c.Nref[1], c.Nref[2]);
    gl.drawArrays(gl.POINTS, 0, 1);
    const pixels = new Uint8Array(4);
    gl.readPixels(0, 0, 1, 1, gl.RGBA, gl.UNSIGNED_BYTE, pixels);
    const result = unpackFloat(pixels);
    const expected = (function () {
      const dot = c.Nref[0] * c.I[0] + c.Nref[1] * c.I[1] + c.Nref[2] * c.I[2];
      return dot < 0.0 ? c.N[0] : -c.N[0];
    })();
    assert.equal(canonicalize(result), canonicalize(expected), `faceforward(${JSON.stringify(c)}) x failed`);
  }
});