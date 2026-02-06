import { test } from 'node:test';
import assert from 'node:assert';
import { webGL2 } from '../../index.js';
import { PACK_FLOAT_GLSL, unpackFloat } from './-core.js';

test('Matrix uniform sanity', async (t) => {
  const gl = await webGL2({ size: { width: 1, height: 1 } });
  const vs = `#version 300 es
        in vec4 position;
        void main() { gl_Position = position; }
    `;
  const fs = `#version 300 es
        precision highp float;
        uniform vec2 u_c0;
        out vec4 outColor;
        void main() {
            ${PACK_FLOAT_GLSL('u_c0.x')}
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
  if (!gl.getProgramParameter(program, gl.LINK_STATUS)) throw new Error(gl.getProgramInfoLog(program));
  gl.useProgram(program);
  const loc = gl.getUniformLocation(program, 'u_c0');
  gl.uniform2f(loc, 1.0, 0.0);
  gl.drawArrays(gl.POINTS, 0, 1);
  const pixels = new Uint8Array(4);
  gl.readPixels(0, 0, 1, 1, gl.RGBA, gl.UNSIGNED_BYTE, pixels);
  const result = unpackFloat(pixels);
  assert.equal(result, 1.0);
});