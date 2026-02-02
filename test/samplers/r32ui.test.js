import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../../index.js';

// R32UI sampler tests — unsigned integer single channel; NEAREST-only

async function compileProgram(gl, vsSource, fsSource) {
  const vs = gl.createShader(gl.VERTEX_SHADER);
  gl.shaderSource(vs, vsSource);
  gl.compileShader(vs);
  if (!gl.getShaderParameter(vs, gl.COMPILE_STATUS)) throw new Error('VS compile: ' + gl.getShaderInfoLog(vs));

  const fs = gl.createShader(gl.FRAGMENT_SHADER);
  gl.shaderSource(fs, fsSource);
  gl.compileShader(fs);
  if (!gl.getShaderParameter(fs, gl.COMPILE_STATUS)) throw new Error('FS compile: ' + gl.getShaderInfoLog(fs));

  const program = gl.createProgram();
  gl.attachShader(program, vs);
  gl.attachShader(program, fs);
  gl.linkProgram(program);
  if (!gl.getProgramParameter(program, gl.LINK_STATUS)) throw new Error('Link failed: ' + gl.getProgramInfoLog(program));

  return program;
}

const VS = `#version 300 es
layout(location = 0) in vec2 a_pos;
layout(location = 1) in vec2 a_uv;
out vec2 v_uv;
void main() { v_uv = a_uv; gl_Position = vec4(a_pos, 0.0, 1.0); }
`;

// Sample unsigned integer texture and output integer uvec4 to integer framebuffer
const FS_UINT_OUT = `#version 300 es
precision highp int;
uniform usampler2D u_tex;
layout(location = 0) out uvec4 fragColor;
void main() {
    fragColor = texelFetch(u_tex, ivec2(0,0), 0);
}
`;

// Helper to read integer pixels (GL_RGBA_INTEGER / GL_UNSIGNED_INT)

test('R32UI sampler tests', async (t) => {
  await t.test('2D NEAREST 1x1 returns exact uint value (integer framebuffer)', async () => {
    const gl = await webGL2();
    try {
      const GL_RGBA32UI = 0x8D70;
      const GL_RGBA_INTEGER = 0x8D9E;
      const GL_UNSIGNED_INT = 0x1405;

      const program = await compileProgram(gl, VS, FS_UINT_OUT);
      gl.useProgram(program);

      // Create 1x1 integer texture with R=42, other channels zero
      const tex = gl.createTexture();
      gl.bindTexture(gl.TEXTURE_2D, tex);
      const src = new Uint32Array([42, 0, 0, 0]);
      gl.texImage2D(gl.TEXTURE_2D, 0, GL_RGBA32UI, 1, 1, 0, GL_RGBA_INTEGER, GL_UNSIGNED_INT, src);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.NEAREST);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.NEAREST);

      // Integer FBO (attach RGBA32UI) and read with GL_RGBA_INTEGER + GL_UNSIGNED_INT
      const dst = gl.createTexture();
      gl.bindTexture(gl.TEXTURE_2D, dst);
      gl.texImage2D(gl.TEXTURE_2D, 0, GL_RGBA32UI, 1, 1, 0, GL_RGBA_INTEGER, GL_UNSIGNED_INT, null);
      const fb = gl.createFramebuffer();
      gl.bindFramebuffer(gl.FRAMEBUFFER, fb);
      gl.framebufferTexture2D(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.TEXTURE_2D, dst, 0);

      // Setup simple quad and draw
      const vertices = new Float32Array([-1, -1, 0, 0, 1, -1, 1, 0, 1, 1, 1, 1, -1, -1, 0, 0, 1, 1, 1, 1, -1, 1, 0, 1]);
      const buf = gl.createBuffer();
      gl.bindBuffer(gl.ARRAY_BUFFER, buf);
      gl.bufferData(gl.ARRAY_BUFFER, vertices, gl.STATIC_DRAW);
      gl.enableVertexAttribArray(0);
      gl.vertexAttribPointer(0, 2, gl.FLOAT, false, 16, 0);
      gl.enableVertexAttribArray(1);
      gl.vertexAttribPointer(1, 2, gl.FLOAT, false, 16, 8);

      const loc = gl.getUniformLocation(program, 'u_tex');
      gl.uniform1i(loc, 0);
      gl.bindTexture(gl.TEXTURE_2D, tex);

      gl.drawArrays(gl.TRIANGLES, 0, 6);

      // Intended readback:
      const out = new Uint32Array(4);
      gl.readPixels(0, 0, 1, 1, GL_RGBA_INTEGER, GL_UNSIGNED_INT, out);

      const actual = { r: out[0], g: out[1], b: out[2], a: out[3] };
      const expected = { r: 42, g: 0, b: 0, a: 0 };
      assert.deepStrictEqual(actual, expected);
    } finally { gl.destroy(); }
  });
});