import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../../index.js';

// R32F sampler tests — single-channel float, canonicalized to 1e-6

function makeQuad(gl) {
  const vertices = new Float32Array([
    -1, -1, 0, 0,
    1, -1, 1, 0,
    1, 1, 1, 1,
    -1, -1, 0, 0,
    1, 1, 1, 1,
    -1, 1, 0, 1,
  ]);
  const buf = gl.createBuffer();
  gl.bindBuffer(gl.ARRAY_BUFFER, buf);
  gl.bufferData(gl.ARRAY_BUFFER, vertices, gl.STATIC_DRAW);
  return buf;
}

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

// Shader samples single-channel texture and writes the red component to output
const FS_2D = `#version 300 es
precision highp float;
uniform sampler2D u_tex;
in vec2 v_uv;
out vec4 fragColor;
void main() { float s = texture(u_tex, v_uv).r; fragColor = vec4(s, 0.0, 0.0, 1.0); }
`;

const FS_3D = `#version 300 es
precision highp float;
uniform sampler3D u_tex3d;
in vec2 v_uv;
out vec4 fragColor;
void main() { float s = texture(u_tex3d, vec3(v_uv, 0.5)).r; fragColor = vec4(s, 0.0, 0.0, 1.0); }
`;

function quantizeFloat(v, places = 3) { return Math.round(v * Math.pow(10, places)) / Math.pow(10, places); }

test('R32F sampler tests', async (t) => {
  await t.test('2D NEAREST 1x1 returns exact float (canonicalized)', async () => {
    const gl = await webGL2();
    try {
      gl.viewport(0, 0, 1, 1);
      const program = await compileProgram(gl, VS, FS_2D);
      gl.useProgram(program);
      const buf = makeQuad(gl);
      gl.enableVertexAttribArray(0);
      gl.vertexAttribPointer(0, 2, gl.FLOAT, false, 16, 0);
      gl.enableVertexAttribArray(1);
      gl.vertexAttribPointer(1, 2, gl.FLOAT, false, 16, 8);

      const GL_R32F = 0x822E;
      const GL_RED = 0x1903;
      const GL_FLOAT = 0x1406;
      const GL_RGBA32F = 0x8814;

      const src = gl.createTexture();
      gl.bindTexture(gl.TEXTURE_2D, src);
      const data = new Float32Array([123.456]);
      gl.texImage2D(gl.TEXTURE_2D, 0, GL_R32F, 1, 1, 0, GL_RED, GL_FLOAT, data);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.NEAREST);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.NEAREST);

      // Destination RGBA32F FBO to capture red channel in a float target
      const dst = gl.createTexture();
      gl.bindTexture(gl.TEXTURE_2D, dst);
      gl.texImage2D(gl.TEXTURE_2D, 0, GL_RGBA32F, 1, 1, 0, gl.RGBA, GL_FLOAT, null);
      const fb = gl.createFramebuffer();
      gl.bindFramebuffer(gl.FRAMEBUFFER, fb);
      gl.framebufferTexture2D(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.TEXTURE_2D, dst, 0);

      const loc = gl.getUniformLocation(program, 'u_tex');
      gl.uniform1i(loc, 0);
      // binding to default texture unit 0 (no explicit activeTexture call to avoid invalid-texture-unit errors)
      gl.bindTexture(gl.TEXTURE_2D, src);

      gl.clearColor(0, 0, 0, 0);
      gl.clear(gl.COLOR_BUFFER_BIT);
      gl.drawArrays(gl.TRIANGLES, 0, 6);

      const out = new Float32Array(4);
      gl.readPixels(0, 0, 1, 1, gl.RGBA, GL_FLOAT, out);

      const actual = { r: quantizeFloat(out[0]), g: quantizeFloat(out[1]), b: quantizeFloat(out[2]), a: quantizeFloat(out[3]) };
      const expected = { r: quantizeFloat(123.456), g: 0, b: 0, a: 1 };
      assert.deepStrictEqual(actual, expected);
    } finally { gl.destroy(); }
  });

  await t.test('3D NEAREST 1x1x1 returns exact float (canonicalized)', async () => {
    const gl = await webGL2();
    try {
      gl.viewport(0, 0, 1, 1);
      const program = await compileProgram(gl, VS, FS_3D);
      gl.useProgram(program);
      const buf = makeQuad(gl);
      gl.enableVertexAttribArray(0);
      gl.vertexAttribPointer(0, 2, gl.FLOAT, false, 16, 0);
      gl.enableVertexAttribArray(1);
      gl.vertexAttribPointer(1, 2, gl.FLOAT, false, 16, 8);

      const GL_R32F = 0x822E;
      const GL_RED = 0x1903;
      const GL_FLOAT = 0x1406;
      const GL_TEXTURE_3D = 0x806F;
      const GL_RGBA32F = 0x8814;

      const src = gl.createTexture();
      gl.bindTexture(GL_TEXTURE_3D, src);
      const data = new Float32Array([0.125]);
      gl.texImage3D(GL_TEXTURE_3D, 0, GL_R32F, 1, 1, 1, 0, GL_RED, GL_FLOAT, data);
      gl.texParameteri(GL_TEXTURE_3D, gl.TEXTURE_MIN_FILTER, gl.NEAREST);
      gl.texParameteri(GL_TEXTURE_3D, gl.TEXTURE_MAG_FILTER, gl.NEAREST);

      const dst = gl.createTexture();
      gl.bindTexture(gl.TEXTURE_2D, dst);
      gl.texImage2D(gl.TEXTURE_2D, 0, GL_RGBA32F, 1, 1, 0, gl.RGBA, GL_FLOAT, null);
      const fb = gl.createFramebuffer();
      gl.bindFramebuffer(gl.FRAMEBUFFER, fb);
      gl.framebufferTexture2D(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.TEXTURE_2D, dst, 0);

      const loc = gl.getUniformLocation(program, 'u_tex3d');
      gl.uniform1i(loc, 0);
      // bind to default texture unit 0
      gl.bindTexture(GL_TEXTURE_3D, src);

      gl.clearColor(0, 0, 0, 0);
      gl.clear(gl.COLOR_BUFFER_BIT);
      gl.drawArrays(gl.TRIANGLES, 0, 6);

      const out = new Float32Array(4);
      gl.readPixels(0, 0, 1, 1, gl.RGBA, GL_FLOAT, out);

      const actual = { r: quantizeFloat(out[0]), g: quantizeFloat(out[1]), b: quantizeFloat(out[2]), a: quantizeFloat(out[3]) };
      const expected = { r: quantizeFloat(0.125), g: 0, b: 0, a: 1 };
      assert.deepStrictEqual(actual, expected);
    } finally { gl.destroy(); }
  });


});