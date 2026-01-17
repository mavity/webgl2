import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('texture sampling works', async () => {
  const gl = await webGL2();
  try {
    // 1. Create a 2x2 texture
    // Red, Green
    // Blue, White
    const tex = gl.createTexture();
    gl.bindTexture(gl.TEXTURE_2D, tex);
    const pixels = new Uint8Array([
      255, 0, 0, 255, 0, 255, 0, 255,
      0, 0, 255, 255, 255, 255, 255, 255
    ]);
    gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, 2, 2, 0, gl.RGBA, gl.UNSIGNED_BYTE, pixels);

    // 2. Create a program that samples the texture
    const vsSource = `#version 300 es
layout(location = 0) in vec4 position;
layout(location = 1) in vec2 uv;
out vec2 v_uv;
void main() {
  v_uv = vec2(0.25, 0.25);
  gl_Position = vec4(0.0, 0.0, 0.0, 1.0);
}
`;

    const fsSource = `#version 300 es
precision highp float;
uniform sampler2D u_texture;
in vec2 v_uv;
out vec4 fragColor;
void main() {
  fragColor = texture(u_texture, v_uv);
}
`;

    const vs = gl.createShader(gl.VERTEX_SHADER);
    gl.shaderSource(vs, vsSource);
    gl.compileShader(vs);

    const fs = gl.createShader(gl.FRAGMENT_SHADER);
    gl.shaderSource(fs, fsSource);
    gl.compileShader(fs);

    const program = gl.createProgram();
    gl.attachShader(program, vs);
    gl.attachShader(program, fs);
    gl.linkProgram(program);
    gl.useProgram(program);

    // 3. Set up geometry (a single point at center, sampling center of texture)
    const posBuf = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, posBuf);
    gl.bufferData(gl.ARRAY_BUFFER, new Float32Array([0.0, 0.0, 0, 1]), gl.STATIC_DRAW);
    gl.enableVertexAttribArray(0);
    gl.vertexAttribPointer(0, 4, gl.FLOAT, false, 0, 0);

    const uvBuf = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, uvBuf);
    gl.bufferData(gl.ARRAY_BUFFER, new Float32Array([0.25, 0.25]), gl.STATIC_DRAW);
    gl.enableVertexAttribArray(1);
    gl.vertexAttribPointer(1, 2, gl.FLOAT, false, 0, 0);

    // 4. Draw
    gl.clearColor(0, 0, 0, 1);
    gl.clear(gl.COLOR_BUFFER_BIT);
    gl.drawArrays(gl.POINTS, 0, 1);

    // 5. Read back the pixel at center (320, 240)
    const result = new Uint8Array(4);
    gl.readPixels(320, 240, 1, 1, gl.RGBA, gl.UNSIGNED_BYTE, result);

    // Center of 2x2 texture with Red, Green, Blue, White should be Green or Blue or something depending on interpolation.
    // But our current implementation is nearest neighbor and clamps.
    // (0.5, 0.5) * (2, 2) = (1, 1).
    // texel_x = 1, texel_y = 1.
    // pixels[1, 1] is White (255, 255, 255, 255).

    assert.strictEqual(result[0], 255, 'Red should be 255');
    assert.strictEqual(result[1], 0, 'Green should be 0');
    assert.strictEqual(result[2], 0, 'Blue should be 0');
    assert.strictEqual(result[3], 255, 'Alpha should be 255');
  } finally {
    gl.destroy();
  }
});

