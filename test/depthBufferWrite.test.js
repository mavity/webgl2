import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('depth buffer writes even when DEPTH_TEST is disabled', async () => {
  const gl = await webGL2();
  try {
    gl.viewport(0, 0, 64, 64);

    // Vertex shader with depth control
    const vsSource = `#version 300 es
    layout(location = 0) in vec2 position;
    layout(location = 1) in float depth;
    
    void main() {
        gl_Position = vec4(position, depth, 1.0);
    }`;

    // Simple fragment shader with solid color
    const fsSource = `#version 300 es
    precision highp float;
    uniform vec4 u_color;
    out vec4 fragColor;
    
    void main() {
        fragColor = u_color;
    }`;

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

    const colorLoc = gl.getUniformLocation(program, 'u_color');

    // Create two overlapping triangles:
    // Triangle 1 (red): closer (z = -0.5, maps to depth ~0.25)
    // Triangle 2 (blue): farther (z = 0.5, maps to depth ~0.75)

    // Triangle 1: Red triangle in the center, closer to camera
    const triangle1 = new Float32Array([
      // x, y, z
      -0.5, -0.5, -0.5,
      0.5, -0.5, -0.5,
      0.0, 0.5, -0.5,
    ]);

    // Triangle 2: Blue triangle in the center, farther from camera
    const triangle2 = new Float32Array([
      -0.5, -0.3, 0.5,
      0.5, -0.3, 0.5,
      0.0, 0.7, 0.5,
    ]);

    const buffer = gl.createBuffer();
    gl.enableVertexAttribArray(0);
    gl.enableVertexAttribArray(1);

    // Clear to black (depth buffer defaults to 1.0)
    gl.clearColor(0.0, 0.0, 0.0, 1.0);
    gl.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT);

    // Phase 1: Draw with DEPTH_TEST **disabled** (default state)
    // Even though testing is disabled, the depth buffer should still be **written**

    // Draw the CLOSER triangle (red, z=-0.5, depth ~0.25)
    gl.bindBuffer(gl.ARRAY_BUFFER, buffer);
    gl.bufferData(gl.ARRAY_BUFFER, triangle1, gl.STATIC_DRAW);
    gl.vertexAttribPointer(0, 2, gl.FLOAT, false, 12, 0);
    gl.vertexAttribPointer(1, 1, gl.FLOAT, false, 12, 8);
    gl.uniform4f(colorLoc, 1.0, 0.0, 0.0, 1.0); // Red
    gl.drawArrays(gl.TRIANGLES, 0, 3);

    // Phase 2: Enable depth testing with GL_GREATER
    // This will render fragments whose depth is GREATER than the current depth buffer value
    gl.enable(gl.DEPTH_TEST);
    gl.depthFunc(gl.GREATER);

    // Draw the FARTHER triangle (blue, z=0.5, depth ~0.75)
    // If Phase 1 wrote depth ~0.25: 0.75 > 0.25 ✓ → blue renders
    // If Phase 1 did NOT write depth (still 1.0): 0.75 > 1.0 ✗ → blue doesn't render
    gl.bindBuffer(gl.ARRAY_BUFFER, buffer);
    gl.bufferData(gl.ARRAY_BUFFER, triangle2, gl.STATIC_DRAW);
    gl.vertexAttribPointer(0, 2, gl.FLOAT, false, 12, 0);
    gl.vertexAttribPointer(1, 1, gl.FLOAT, false, 12, 8);
    gl.uniform4f(colorLoc, 0.0, 0.0, 1.0, 1.0); // Blue
    gl.drawArrays(gl.TRIANGLES, 0, 3);

    // Read pixels from center of the viewport
    const pixels = new Uint8Array(64 * 64 * 4);
    gl.readPixels(0, 0, 64, 64, gl.RGBA, gl.UNSIGNED_BYTE, pixels);

    // Sample pixel at the center (32, 32) - should be RED if depth buffer is working
    // Even though DEPTH_TEST is disabled, the depth buffer should still be written,
    // so the closer red triangle should overwrite the farther blue triangle
    const centerX = 32;
    const centerY = 32;
    const idx = (centerY * 64 + centerX) * 4;

    const r = pixels[idx];
    const g = pixels[idx + 1];
    const b = pixels[idx + 2];

    // The center should be blue if depth buffer was written in Phase 1
    // If depth wasn't written, the blue triangle would fail GL_GREATER test and we'd see red
    assert.ok(b > 200, `Center pixel should be blue (b=${b}), got RGB(${r}, ${g}, ${b}). This means Phase 1 wrote depth correctly.`);
    assert.ok(r < 50, `Center pixel should not have red (r=${r}), got RGB(${r}, ${g}, ${b})`);
    assert.ok(g < 50, `Center pixel should not have green (g=${g}), got RGB(${r}, ${g}, ${b})`);

    // Additional verification: check pixels in the overlapping region
    let redCount = 0;
    let blueCount = 0;

    for (let y = 24; y < 40; y++) {
      for (let x = 24; x < 40; x++) {
        const idx = (y * 64 + x) * 4;
        const r = pixels[idx];
        const b = pixels[idx + 2];

        if (r > 200 && b < 50) redCount++;
        if (b > 200 && r < 50) blueCount++;
      }
    }

    // The overlapping region should be predominantly blue (farther triangle rendered with GL_GREATER)
    assert.ok(blueCount > redCount,
      `Overlapping region should be mostly blue (depth was written). Blue pixels: ${blueCount}, Red pixels: ${redCount}`);

  } finally {
    gl.destroy();
  }
});
