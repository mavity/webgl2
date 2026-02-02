import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../../index.js';

// RGBA8 (UNORM) sampler tests — named-key assertions

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
precision mediump float;
uniform sampler2D u_tex;
in vec2 v_uv;
out vec4 fragColor;
void main() { fragColor = texture(u_tex, v_uv); }
`;

const FS_3D = `#version 300 es
precision mediump float;
uniform sampler3D u_tex3d;
in vec2 v_uv;
out vec4 fragColor;
void main() { fragColor = texture(u_tex3d, vec3(v_uv, 0.5)); }
`;

test('RGBA8 (UNORM) sampler tests', async (t) => {
  await t.test('2D NEAREST 1x1 returns exact byte values (named-keys)', async () => {
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

      const tex = gl.createTexture();
      gl.bindTexture(gl.TEXTURE_2D, tex);
      const src = new Uint8Array([10, 20, 30, 255]);
      gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, 1, 1, 0, gl.RGBA, gl.UNSIGNED_BYTE, src);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.NEAREST);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.NEAREST);

      const loc = gl.getUniformLocation(program, 'u_tex');
      gl.uniform1i(loc, 0);

      gl.clearColor(0, 0, 0, 0);
      gl.clear(gl.COLOR_BUFFER_BIT);
      gl.drawArrays(gl.TRIANGLES, 0, 6);

      const out = new Uint8Array(4);
      gl.readPixels(0, 0, 1, 1, gl.RGBA, gl.UNSIGNED_BYTE, out);

      const actual = { r: out[0], g: out[1], b: out[2], a: out[3] };
      const expected = { r: 10, g: 20, b: 30, a: 255 };
      assert.deepStrictEqual(actual, expected);
    } finally { gl.destroy(); }
  });

  await t.test('2D LINEAR 2x1 blends to expected value (named-keys)', async () => {
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

      const tex = gl.createTexture();
      gl.bindTexture(gl.TEXTURE_2D, tex);
      const data = new Uint8Array([0, 0, 0, 255, 128, 0, 0, 255]);
      gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, 2, 1, 0, gl.RGBA, gl.UNSIGNED_BYTE, data);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);

      const loc = gl.getUniformLocation(program, 'u_tex');
      gl.uniform1i(loc, 0);

      gl.clearColor(0, 0, 0, 0);
      gl.clear(gl.COLOR_BUFFER_BIT);
      gl.drawArrays(gl.TRIANGLES, 0, 6);

      const out = new Uint8Array(4);
      gl.readPixels(0, 0, 1, 1, gl.RGBA, gl.UNSIGNED_BYTE, out);

      const actual = { r: out[0], g: out[1], b: out[2], a: out[3] };
      const expected = { r: 64, g: 0, b: 0, a: 255 };
      assert.deepStrictEqual(actual, expected);
    } finally { gl.destroy(); }
  });

  await t.test('3D NEAREST 1x1x1 returns exact byte values (named-keys)', async () => {
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

      const tex = gl.createTexture();
      gl.bindTexture(gl.TEXTURE_3D, tex);
      const src = new Uint8Array([200, 150, 100, 255]);
      gl.texImage3D(gl.TEXTURE_3D, 0, gl.RGBA, 1, 1, 1, 0, gl.RGBA, gl.UNSIGNED_BYTE, src);
      gl.texParameteri(gl.TEXTURE_3D, gl.TEXTURE_MIN_FILTER, gl.NEAREST);
      gl.texParameteri(gl.TEXTURE_3D, gl.TEXTURE_MAG_FILTER, gl.NEAREST);

      const loc = gl.getUniformLocation(program, 'u_tex3d');
      gl.uniform1i(loc, 0);

      gl.clearColor(0, 0, 0, 0);
      gl.clear(gl.COLOR_BUFFER_BIT);
      gl.drawArrays(gl.TRIANGLES, 0, 6);

      const out = new Uint8Array(4);
      gl.readPixels(0, 0, 1, 1, gl.RGBA, gl.UNSIGNED_BYTE, out);

      const actual = { r: out[0], g: out[1], b: out[2], a: out[3] };
      const expected = { r: 200, g: 150, b: 100, a: 255 };
      assert.deepStrictEqual(actual, expected);
    } finally { gl.destroy(); }
  });

  await t.test('Wrap modes: REPEAT vs CLAMP_TO_EDGE behave as expected (2D)', async () => {
    const gl = await webGL2();
    try {
      gl.viewport(0, 0, 1, 1);
      const FS_WRAP = `#version 300 es
precision mediump float;
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

      const tex = gl.createTexture();
      gl.bindTexture(gl.TEXTURE_2D, tex);
      const data = new Uint8Array([100,0,0,255, 0,200,0,255]); // x=0 => red, x=1 => green
      gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, 2,1,0, gl.RGBA, gl.UNSIGNED_BYTE, data);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.NEAREST);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.NEAREST);

      // REPEAT: 1.5 maps to 0.5 -> should sample x=1 (green)
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.REPEAT);
      const loc = gl.getUniformLocation(program, 'u_tex');
      gl.uniform1i(loc, 0);
      gl.clearColor(0,0,0,0);
      gl.clear(gl.COLOR_BUFFER_BIT);
      gl.drawArrays(gl.TRIANGLES, 0, 6);
      const out1 = new Uint8Array(4);
      gl.readPixels(0,0,1,1, gl.RGBA, gl.UNSIGNED_BYTE, out1);
      const actual1 = { r: out1[0], g: out1[1], b: out1[2], a: out1[3] };
      const expected1 = { r: 0, g: 200, b: 0, a: 255 };
      assert.deepStrictEqual(actual1, expected1);

    } finally { gl.destroy(); }
  });

  await t.test('Wrap CLAMP_TO_EDGE samples edge texel for out-of-bounds coords (2D)', async () => {
    const gl = await webGL2();
    try {
      gl.viewport(0, 0, 1, 1);
      const FS_CLAMP = `#version 300 es
precision mediump float;
uniform sampler2D u_tex;
in vec2 v_uv;
out vec4 fragColor;
void main() { fragColor = texture(u_tex, v_uv - vec2(0.5, 0.0)); }
`;
      const program = await compileProgram(gl, VS, FS_CLAMP);
      gl.useProgram(program);
      const buf = makeQuad(gl);
      gl.enableVertexAttribArray(0);
      gl.vertexAttribPointer(0, 2, gl.FLOAT, false, 16, 0);
      gl.enableVertexAttribArray(1);
      gl.vertexAttribPointer(1, 2, gl.FLOAT, false, 16, 8);

      const tex = gl.createTexture();
      gl.bindTexture(gl.TEXTURE_2D, tex);
      const data = new Uint8Array([100,0,0,255, 0,200,0,255]); // x=0 => red, x=1 => green
      gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, 2,1,0, gl.RGBA, gl.UNSIGNED_BYTE, data);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.NEAREST);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.NEAREST);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);

      const loc = gl.getUniformLocation(program, 'u_tex');
      gl.uniform1i(loc, 0);

      gl.clearColor(0,0,0,0);
      gl.clear(gl.COLOR_BUFFER_BIT);
      gl.drawArrays(gl.TRIANGLES, 0, 6);

      const out = new Uint8Array(4);
      gl.readPixels(0,0,1,1, gl.RGBA, gl.UNSIGNED_BYTE, out);
      const actual = { r: out[0], g: out[1], b: out[2], a: out[3] };
      const expected = { r: 100, g: 0, b: 0, a: 255 };
      assert.deepStrictEqual(actual, expected);
    } finally { gl.destroy(); }
  });
});