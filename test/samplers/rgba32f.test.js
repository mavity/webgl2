import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../../index.js';

// RGBA32F sampler tests — float values canonicalized to 1e-6

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

const FS_2D = `#version 300 es
precision highp float;
uniform sampler2D u_tex;
in vec2 v_uv;
out vec4 fragColor;
void main() { fragColor = texture(u_tex, v_uv); }
`;

const FS_3D = `#version 300 es
precision highp float;
uniform sampler3D u_tex3d;
in vec2 v_uv;
out vec4 fragColor;
void main() { fragColor = texture(u_tex3d, vec3(v_uv, 0.5)); }
`;

function quantizeFloat(v, places = 6) { return Math.round(v * Math.pow(10, places)) / Math.pow(10, places); }

function canonicalizeOutFloat(arr) {
  return { r: quantizeFloat(arr[0]), g: quantizeFloat(arr[1]), b: quantizeFloat(arr[2]), a: quantizeFloat(arr[3]) };
}

test('RGBA32F sampler tests', async (t) => {
  await t.test('2D NEAREST 1x1 returns exact float values (canonicalized)', async () => {
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

      const GL_RGBA32F = 0x8814;
      const GL_RGBA = 0x1908;
      const GL_FLOAT = 0x1406;

      const src = gl.createTexture();
      gl.bindTexture(gl.TEXTURE_2D, src);
      const data = new Float32Array([1.25, 2.5, -0.5, 0.125]);
      gl.texImage2D(gl.TEXTURE_2D, 0, GL_RGBA32F, 1, 1, 0, GL_RGBA, GL_FLOAT, data);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.NEAREST);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.NEAREST);

      // render to float FBO
      const dst = gl.createTexture();
      gl.bindTexture(gl.TEXTURE_2D, dst);
      gl.texImage2D(gl.TEXTURE_2D, 0, GL_RGBA32F, 1, 1, 0, GL_RGBA, GL_FLOAT, null);
      const fb = gl.createFramebuffer();
      gl.bindFramebuffer(gl.FRAMEBUFFER, fb);
      gl.framebufferTexture2D(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.TEXTURE_2D, dst, 0);

      const loc = gl.getUniformLocation(program, 'u_tex');
      gl.uniform1i(loc, 0);

      // bind to default texture unit 0
      gl.bindTexture(gl.TEXTURE_2D, src);
      gl.clearColor(0, 0, 0, 0);
      gl.clear(gl.COLOR_BUFFER_BIT);
      gl.drawArrays(gl.TRIANGLES, 0, 6);

      const out = new Float32Array(4);
      gl.readPixels(0, 0, 1, 1, GL_RGBA, GL_FLOAT, out);

      const actual = canonicalizeOutFloat(out);
      const expected = { r: 1.25, g: 2.5, b: -0.5, a: 0.125 };
      assert.deepStrictEqual(actual, expected);
    } finally { gl.destroy(); }
  });

  await t.test('2D LINEAR 2x1 blends to expected float value (canonicalized)', async () => {
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

      const GL_RGBA32F = 0x8814;
      const GL_RGBA = 0x1908;
      const GL_FLOAT = 0x1406;

      const src = gl.createTexture();
      gl.bindTexture(gl.TEXTURE_2D, src);
      const data = new Float32Array([0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0]);
      gl.texImage2D(gl.TEXTURE_2D, 0, GL_RGBA32F, 2, 1, 0, GL_RGBA, GL_FLOAT, data);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);

      const dst = gl.createTexture();
      gl.bindTexture(gl.TEXTURE_2D, dst);
      gl.texImage2D(gl.TEXTURE_2D, 0, GL_RGBA32F, 1, 1, 0, GL_RGBA, GL_FLOAT, null);
      const fb = gl.createFramebuffer();
      gl.bindFramebuffer(gl.FRAMEBUFFER, fb);
      gl.framebufferTexture2D(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.TEXTURE_2D, dst, 0);

      const loc = gl.getUniformLocation(program, 'u_tex');
      gl.uniform1i(loc, 0);

      // bind to default texture unit 0
      gl.bindTexture(gl.TEXTURE_2D, src);
      gl.clearColor(0, 0, 0, 0);
      gl.clear(gl.COLOR_BUFFER_BIT);
      gl.drawArrays(gl.TRIANGLES, 0, 6);

      const out = new Float32Array(4);
      gl.readPixels(0, 0, 1, 1, GL_RGBA, GL_FLOAT, out);

      const actual = canonicalizeOutFloat(out);
      const expected = { r: 0.5, g: 0, b: 0, a: 1.0 };
      assert.deepStrictEqual(actual, expected);
    } finally { gl.destroy(); }
  });

  await t.test('3D NEAREST 1x1x1 returns exact float values (canonicalized)', async () => {
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

      const GL_RGBA32F = 0x8814;
      const GL_RGBA = 0x1908;
      const GL_FLOAT = 0x1406;
      const GL_TEXTURE_3D = 0x806F;

      const src = gl.createTexture();
      gl.bindTexture(GL_TEXTURE_3D, src);
      const data = new Float32Array([2.0, 4.0, 6.0, 8.0]);
      gl.texImage3D(GL_TEXTURE_3D, 0, GL_RGBA32F, 1, 1, 1, 0, GL_RGBA, GL_FLOAT, data);
      gl.texParameteri(GL_TEXTURE_3D, gl.TEXTURE_MIN_FILTER, gl.NEAREST);
      gl.texParameteri(GL_TEXTURE_3D, gl.TEXTURE_MAG_FILTER, gl.NEAREST);

      const dst = gl.createTexture();
      gl.bindTexture(gl.TEXTURE_2D, dst);
      gl.texImage2D(gl.TEXTURE_2D, 0, GL_RGBA32F, 1, 1, 0, GL_RGBA, GL_FLOAT, null);
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
      gl.readPixels(0, 0, 1, 1, GL_RGBA, GL_FLOAT, out);

      const actual = canonicalizeOutFloat(out);
      const expected = { r: 2.0, g: 4.0, b: 6.0, a: 8.0 };
      assert.deepStrictEqual(actual, expected);
    } finally { gl.destroy(); }
  });


  await t.test('Wrap mode REPEAT works for float textures (2D)', async () => {
    const gl = await webGL2();
    try {
      gl.viewport(0, 0, 1, 1);
      const FS_WRAP = `#version 300 es
precision highp float;
uniform sampler2D u_tex;
in vec2 v_uv;
out vec4 fragColor;
void main() { fragColor = texture(u_tex, v_uv * 1.5); }
`;
      const program = await compileProgram(gl, VS, FS_WRAP);
      gl.useProgram(program);
      const buf = makeQuad(gl);
      gl.enableVertexAttribArray(0);
      gl.vertexAttribPointer(0, 2, gl.FLOAT, false, 16, 0);
      gl.enableVertexAttribArray(1);
      gl.vertexAttribPointer(1, 2, gl.FLOAT, false, 16, 8);

      const GL_RGBA32F = 0x8814;
      const GL_RGBA = 0x1908;
      const GL_FLOAT = 0x1406;

      const tex = gl.createTexture();
      gl.bindTexture(gl.TEXTURE_2D, tex);
      const data = new Float32Array([1.0,0,0,1, 0,1.0,0,1]);
      gl.texImage2D(gl.TEXTURE_2D, 0, GL_RGBA32F, 2, 1, 0, GL_RGBA, GL_FLOAT, data);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.NEAREST);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.NEAREST);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.REPEAT);

      const loc = gl.getUniformLocation(program, 'u_tex');
      gl.uniform1i(loc, 0);

      gl.clearColor(0,0,0,0);
      gl.clear(gl.COLOR_BUFFER_BIT);
      gl.drawArrays(gl.TRIANGLES, 0, 6);

      const out = new Float32Array(4);
      gl.readPixels(0,0,1,1, GL_RGBA, GL_FLOAT, out);
      const actual = canonicalizeOutFloat(out);
      const expected = { r: 0.0, g: 1.0, b: 0.0, a: 1.0 };
      assert.deepStrictEqual(actual, expected);
    } finally { gl.destroy(); }
  });
});