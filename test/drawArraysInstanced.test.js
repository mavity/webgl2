import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('drawArraysInstanced draws multiple instances with divisor', async () => {
  const gl = await webGL2();
  try {
    const vs = gl.createShader(gl.VERTEX_SHADER);
    gl.shaderSource(vs, `#version 300 es
      layout(location=0) in vec2 position;
      layout(location=1) in float offset;
      void main() {
        gl_Position = vec4(position.x + offset, position.y, 0.0, 1.0);
        gl_PointSize = 10.0;
      }
    `);
    gl.compileShader(vs);
    if (!gl.getShaderParameter(vs, gl.COMPILE_STATUS)) {
      throw new Error(gl.getShaderInfoLog(vs));
    }

    const fs = gl.createShader(gl.FRAGMENT_SHADER);
    gl.shaderSource(fs, `#version 300 es
      precision mediump float;
      out vec4 color;
      void main() {
        color = vec4(1.0, 0.0, 0.0, 1.0);
      }
    `);
    gl.compileShader(fs);
    if (!gl.getShaderParameter(fs, gl.COMPILE_STATUS)) {
      throw new Error(gl.getShaderInfoLog(fs));
    }

    const prog = gl.createProgram();
    gl.attachShader(prog, vs);
    gl.attachShader(prog, fs);
    gl.linkProgram(prog);
    if (!gl.getProgramParameter(prog, gl.LINK_STATUS)) {
      throw new Error(gl.getProgramInfoLog(prog));
    }
    gl.useProgram(prog);

    // Vertex buffer: Single point at (-0.5, 0.0)
    const buf = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, buf);
    gl.bufferData(gl.ARRAY_BUFFER, new Float32Array([-0.5, 0.0]), gl.STATIC_DRAW);

    gl.enableVertexAttribArray(0);
    gl.vertexAttribPointer(0, 2, gl.FLOAT, false, 0, 0);

    // Instance buffer: Offsets [0.0, 0.5]
    const instanceBuf = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, instanceBuf);
    gl.bufferData(gl.ARRAY_BUFFER, new Float32Array([0.0, 0.5]), gl.STATIC_DRAW);

    gl.enableVertexAttribArray(1);
    gl.vertexAttribPointer(1, 1, gl.FLOAT, false, 0, 0);
    gl.vertexAttribDivisor(1, 1); // Advance once per instance

    gl.viewport(0, 0, 100, 100);
    gl.clearColor(0, 0, 0, 0);
    gl.clear(gl.COLOR_BUFFER_BIT);

    // Draw 2 instances.
    // Instance 0: (-0.5 + 0.0, 0.0) = (-0.5, 0.0) -> Screen left
    // Instance 1: (-0.5 + 0.5, 0.0) = (0.0, 0.0) -> Screen center
    gl.drawArraysInstanced(gl.POINTS, 0, 1, 2);

    const pixels = new Uint8Array(4);
    
    // Check instance 0 (left side)
    gl.readPixels(25, 50, 1, 1, gl.RGBA, gl.UNSIGNED_BYTE, pixels);
    assert.deepEqual([pixels[0], pixels[1], pixels[2], pixels[3]], [255, 0, 0, 255], 'Instance 0 drawn');

    // Check instance 1 (center)
    gl.readPixels(50, 50, 1, 1, gl.RGBA, gl.UNSIGNED_BYTE, pixels);
    assert.deepEqual([pixels[0], pixels[1], pixels[2], pixels[3]], [255, 0, 0, 255], 'Instance 1 drawn');

  } finally {
    gl.destroy();
  }
});
