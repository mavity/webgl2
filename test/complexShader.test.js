import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('Complex shader with internal functions and multiple uniforms', async () => {
  const gl = await webGL2();
  try {
    const vs = gl.createShader(gl.VERTEX_SHADER);
    gl.shaderSource(vs, `#version 300 es
layout(location = 0) in vec4 pos;
uniform float scale;
uniform vec2 offset;

float get_scale() {
    return scale * 2.0;
}

vec4 transform(vec4 p) {
    float s = get_scale();
    return vec4(p.xy * s + offset, p.zw);
}

void main() {
    gl_Position = transform(pos);
}`);
    gl.compileShader(vs);

    const fs = gl.createShader(gl.FRAGMENT_SHADER);
    gl.shaderSource(fs, `#version 300 es
precision highp float;
out vec4 color;
uniform vec3 baseColor;

vec3 get_color() {
    return baseColor * 0.5;
}

void main() {
    color = vec4(get_color() + 0.5, 1.0);
}`);
    gl.compileShader(fs);

    const prog = gl.createProgram();
    gl.attachShader(prog, vs);
    gl.attachShader(prog, fs);
    gl.linkProgram(prog);
    gl.useProgram(prog);

    // Set uniforms
    const locScale = gl.getUniformLocation(prog, 'scale');
    const locOffset = gl.getUniformLocation(prog, 'offset');
    const locBaseColor = gl.getUniformLocation(prog, 'baseColor');

    gl.uniform1f(locScale, 0.5); // effective scale = 1.0
    gl.uniform2f(locOffset, 0.1, 0.2);
    gl.uniform3f(locBaseColor, 1.0, 0.0, 1.0); // effective color = (0.5+0.5, 0+0.5, 0.5+0.5) = (1.0, 0.5, 1.0)

    // Buffer with one point at (0,0)
    const buf = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, buf);
    gl.bufferData(gl.ARRAY_BUFFER, new Float32Array([0.0, 0.0, 0.0, 1.0]), gl.STATIC_DRAW);
    gl.enableVertexAttribArray(0);
    gl.vertexAttribPointer(0, 4, gl.FLOAT, false, 0, 0);

    gl.clearColor(0, 0, 0, 1);
    gl.clear(gl.COLOR_BUFFER_BIT);

    gl.drawArrays(gl.POINTS, 0, 1);

    // Expected position: (0,0) * 1.0 + (0.1, 0.2) = (0.1, 0.2) NDC
    // (0.1, 0.2) NDC -> (320 + 32, 240 + 48) = (352, 288)
    const pixels = new Uint8Array(4);
    gl.readPixels(352, 288, 1, 1, gl.RGBA, gl.UNSIGNED_BYTE, pixels);

    assert.strictEqual(pixels[0], 255, 'Red should be 255');
    assert.strictEqual(pixels[1], 127, 'Green should be 127');
    assert.strictEqual(pixels[2], 255, 'Blue should be 255');
    assert.strictEqual(pixels[3], 255, 'Alpha should be 255');

  } finally {
    gl.destroy();
  }
});
