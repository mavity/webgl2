import test from 'node:test';
import assert from 'node:assert';
import { webGL2 } from '../index.js';

/**
 * Helper function for comparing pixel values with tolerance
 * @param {Uint8Array} actual - Actual pixel values
 * @param {number[]} expected - Expected pixel values
 * @param {number} tolerance - Allowed difference per component (default: 3)
 * @param {string} message - Error message prefix
 */
function assertPixelsEqual(actual, expected, tolerance = 3, message = 'Pixel mismatch') {
  assert.strictEqual(actual.length, expected.length, `${message}: array length mismatch`);
  for (let i = 0; i < actual.length; i++) {
    const diff = Math.abs(actual[i] - expected[i]);
    assert.ok(
      diff <= tolerance,
      `${message}: component ${i} expected ~${expected[i]}, got ${actual[i]} (diff: ${diff}, tolerance: ${tolerance})`
    );
  }
}

test('Vertex shader - Position only (single value return)', async () => {
  const gl = await webGL2();
  try {
    gl.viewport(0, 0, 64, 64);
    // Minimal vertex shader that only outputs position (WGSL Case B equivalent)
    const vs = `#version 300 es
      void main() { 
        gl_Position = vec4(0.0, 0.0, 0.0, 1.0); 
      }`;
    const fs = `#version 300 es
      precision highp float; 
      out vec4 fragColor; 
      void main() { 
        fragColor = vec4(1.0, 0.0, 0.0, 1.0); 
      }`;
    
    const s_vs = gl.createShader(gl.VERTEX_SHADER);
    gl.shaderSource(s_vs, vs);
    gl.compileShader(s_vs);
    assert.ok(gl.getShaderParameter(s_vs, gl.COMPILE_STATUS), 'Vertex shader compile failed');
    
    const s_fs = gl.createShader(gl.FRAGMENT_SHADER);
    gl.shaderSource(s_fs, fs);
    gl.compileShader(s_fs);
    assert.ok(gl.getShaderParameter(s_fs, gl.COMPILE_STATUS), 'Fragment shader compile failed');
    
    const prog = gl.createProgram();
    gl.attachShader(prog, s_vs);
    gl.attachShader(prog, s_fs);
    gl.linkProgram(prog);
    assert.ok(gl.getProgramParameter(prog, gl.LINK_STATUS), 'Program link failed');
    gl.useProgram(prog);
    
    gl.clearColor(0, 0, 0, 1);
    gl.clear(gl.COLOR_BUFFER_BIT);
    gl.drawArrays(gl.POINTS, 0, 1);
    
    const pixels = new Uint8Array(4);
    gl.readPixels(32, 32, 1, 1, gl.RGBA, gl.UNSIGNED_BYTE, pixels);
    assert.deepStrictEqual(Array.from(pixels), [255, 0, 0, 255], 'Expected red pixel');
  } finally {
    gl.destroy();
  }
});

test('Vertex shader - Position with varyings (struct return)', async () => {
  const gl = await webGL2();
  try {
    gl.viewport(0, 0, 64, 64);
    // Vertex shader with position and varying output
    const vs = `#version 300 es
      out vec3 vColor;
      void main() { 
        gl_Position = vec4(0.0, 0.0, 0.0, 1.0);
        gl_PointSize = 1.0;
        vColor = vec3(0.0, 1.0, 0.0);
      }`;
    const fs = `#version 300 es
      precision highp float;
      in vec3 vColor;
      out vec4 fragColor; 
      void main() { 
        fragColor = vec4(vColor, 1.0); 
      }`;
    
    const s_vs = gl.createShader(gl.VERTEX_SHADER);
    gl.shaderSource(s_vs, vs);
    gl.compileShader(s_vs);
    assert.ok(gl.getShaderParameter(s_vs, gl.COMPILE_STATUS), 'Vertex shader compile failed');
    
    const s_fs = gl.createShader(gl.FRAGMENT_SHADER);
    gl.shaderSource(s_fs, fs);
    gl.compileShader(s_fs);
    assert.ok(gl.getShaderParameter(s_fs, gl.COMPILE_STATUS), 'Fragment shader compile failed');
    
    const prog = gl.createProgram();
    gl.attachShader(prog, s_vs);
    gl.attachShader(prog, s_fs);
    gl.linkProgram(prog);
    assert.ok(gl.getProgramParameter(prog, gl.LINK_STATUS), 'Program link failed');
    gl.useProgram(prog);
    
    gl.clearColor(0, 0, 0, 1);
    gl.clear(gl.COLOR_BUFFER_BIT);
    gl.drawArrays(gl.POINTS, 0, 1);
    
    const pixels = new Uint8Array(4);
    gl.readPixels(32, 32, 1, 1, gl.RGBA, gl.UNSIGNED_BYTE, pixels);
    assert.deepStrictEqual(Array.from(pixels), [0, 255, 0, 255], 'Expected green pixel from varying');
  } finally {
    gl.destroy();
  }
});

test('Fragment shader - Single color output', async () => {
  const gl = await webGL2();
  try {
    gl.viewport(0, 0, 64, 64);
    const vs = `#version 300 es
      void main() { 
        gl_Position = vec4(0.0, 0.0, 0.0, 1.0);
        gl_PointSize = 1.0;
      }`;
    // Fragment shader with single color output at location 0
    const fs = `#version 300 es
      precision highp float;
      out vec4 fragColor;
      void main() { 
        fragColor = vec4(0.0, 0.0, 1.0, 1.0);
      }`;
    
    const s_vs = gl.createShader(gl.VERTEX_SHADER);
    gl.shaderSource(s_vs, vs);
    gl.compileShader(s_vs);
    assert.ok(gl.getShaderParameter(s_vs, gl.COMPILE_STATUS), 'Vertex shader compile failed');
    
    const s_fs = gl.createShader(gl.FRAGMENT_SHADER);
    gl.shaderSource(s_fs, fs);
    gl.compileShader(s_fs);
    assert.ok(gl.getShaderParameter(s_fs, gl.COMPILE_STATUS), 'Fragment shader compile failed');
    
    const prog = gl.createProgram();
    gl.attachShader(prog, s_vs);
    gl.attachShader(prog, s_fs);
    gl.linkProgram(prog);
    assert.ok(gl.getProgramParameter(prog, gl.LINK_STATUS), 'Program link failed');
    gl.useProgram(prog);
    
    gl.clearColor(0, 0, 0, 1);
    gl.clear(gl.COLOR_BUFFER_BIT);
    gl.drawArrays(gl.POINTS, 0, 1);
    
    const pixels = new Uint8Array(4);
    gl.readPixels(32, 32, 1, 1, gl.RGBA, gl.UNSIGNED_BYTE, pixels);
    assert.deepStrictEqual(Array.from(pixels), [0, 0, 255, 255], 'Expected blue pixel');
  } finally {
    gl.destroy();
  }
});

test('Fragment shader - Multiple color outputs', async () => {
  const gl = await webGL2();
  try {
    gl.viewport(0, 0, 64, 64);
    const vs = `#version 300 es
      void main() { 
        gl_Position = vec4(0.0, 0.0, 0.0, 1.0);
        gl_PointSize = 1.0;
      }`;
    // Fragment shader with multiple color outputs (MRT)
    const fs = `#version 300 es
      precision highp float;
      layout(location = 0) out vec4 color0;
      layout(location = 1) out vec4 color1;
      void main() { 
        color0 = vec4(1.0, 0.0, 0.0, 1.0);
        color1 = vec4(0.0, 1.0, 0.0, 1.0);
      }`;
    
    const s_vs = gl.createShader(gl.VERTEX_SHADER);
    gl.shaderSource(s_vs, vs);
    gl.compileShader(s_vs);
    assert.ok(gl.getShaderParameter(s_vs, gl.COMPILE_STATUS), 'Vertex shader compile failed');
    
    const s_fs = gl.createShader(gl.FRAGMENT_SHADER);
    gl.shaderSource(s_fs, fs);
    gl.compileShader(s_fs);
    assert.ok(gl.getShaderParameter(s_fs, gl.COMPILE_STATUS), 'Fragment shader compile failed');
    
    const prog = gl.createProgram();
    gl.attachShader(prog, s_vs);
    gl.attachShader(prog, s_fs);
    gl.linkProgram(prog);
    assert.ok(gl.getProgramParameter(prog, gl.LINK_STATUS), 'Program link failed');
    gl.useProgram(prog);
    
    // Note: Full MRT testing requires framebuffer setup
    // For now, just verify the shader compiles and links
    gl.clearColor(0, 0, 0, 1);
    gl.clear(gl.COLOR_BUFFER_BIT);
    gl.drawArrays(gl.POINTS, 0, 1);
    
    // Should render color0 (red) to default framebuffer
    const pixels = new Uint8Array(4);
    gl.readPixels(32, 32, 1, 1, gl.RGBA, gl.UNSIGNED_BYTE, pixels);
    assert.deepStrictEqual(Array.from(pixels), [255, 0, 0, 255], 'Expected red pixel from color0');
  } finally {
    gl.destroy();
  }
});

test('Fragment shader - Depth output (if supported)', async () => {
  const gl = await webGL2();
  try {
    gl.viewport(0, 0, 64, 64);
    const vs = `#version 300 es
      void main() { 
        gl_Position = vec4(0.0, 0.0, 0.0, 1.0);
        gl_PointSize = 1.0;
      }`;
    // Fragment shader that writes to gl_FragDepth
    const fs = `#version 300 es
      precision highp float;
      out vec4 fragColor;
      void main() { 
        gl_FragDepth = 0.5;
        fragColor = vec4(1.0, 1.0, 0.0, 1.0);
      }`;
    
    const s_vs = gl.createShader(gl.VERTEX_SHADER);
    gl.shaderSource(s_vs, vs);
    gl.compileShader(s_vs);
    assert.ok(gl.getShaderParameter(s_vs, gl.COMPILE_STATUS), 'Vertex shader compile failed');
    
    const s_fs = gl.createShader(gl.FRAGMENT_SHADER);
    gl.shaderSource(s_fs, fs);
    gl.compileShader(s_fs);
    assert.ok(gl.getShaderParameter(s_fs, gl.COMPILE_STATUS), 'Fragment shader compile failed');
    
    const prog = gl.createProgram();
    gl.attachShader(prog, s_vs);
    gl.attachShader(prog, s_fs);
    gl.linkProgram(prog);
    assert.ok(gl.getProgramParameter(prog, gl.LINK_STATUS), 'Program link failed');
    gl.useProgram(prog);
    
    gl.enable(gl.DEPTH_TEST);
    gl.clearColor(0, 0, 0, 1);
    gl.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT);
    gl.drawArrays(gl.POINTS, 0, 1);
    
    const pixels = new Uint8Array(4);
    gl.readPixels(32, 32, 1, 1, gl.RGBA, gl.UNSIGNED_BYTE, pixels);
    assert.deepStrictEqual(Array.from(pixels), [255, 255, 0, 255], 'Expected yellow pixel');
  } finally {
    gl.destroy();
  }
});

test('Vertex shader - Multiple varyings of different types', async () => {
  const gl = await webGL2();
  try {
    gl.viewport(0, 0, 64, 64);
    // Vertex shader with multiple varyings
    const vs = `#version 300 es
      out vec3 vColor;
      out vec2 vTexCoord;
      out float vIntensity;
      void main() { 
        gl_Position = vec4(0.0, 0.0, 0.0, 1.0);
        gl_PointSize = 1.0;
        vColor = vec3(1.0, 0.5, 0.0);
        vTexCoord = vec2(0.5, 0.5);
        vIntensity = 0.8;
      }`;
    const fs = `#version 300 es
      precision highp float;
      in vec3 vColor;
      in vec2 vTexCoord;
      in float vIntensity;
      out vec4 fragColor;
      void main() { 
        fragColor = vec4(vColor * vIntensity, 1.0);
      }`;
    
    const s_vs = gl.createShader(gl.VERTEX_SHADER);
    gl.shaderSource(s_vs, vs);
    gl.compileShader(s_vs);
    assert.ok(gl.getShaderParameter(s_vs, gl.COMPILE_STATUS), 'Vertex shader compile failed');
    
    const s_fs = gl.createShader(gl.FRAGMENT_SHADER);
    gl.shaderSource(s_fs, fs);
    gl.compileShader(s_fs);
    assert.ok(gl.getShaderParameter(s_fs, gl.COMPILE_STATUS), 'Fragment shader compile failed');
    
    const prog = gl.createProgram();
    gl.attachShader(prog, s_vs);
    gl.attachShader(prog, s_fs);
    gl.linkProgram(prog);
    assert.ok(gl.getProgramParameter(prog, gl.LINK_STATUS), 'Program link failed');
    gl.useProgram(prog);
    
    gl.clearColor(0, 0, 0, 1);
    gl.clear(gl.COLOR_BUFFER_BIT);
    gl.drawArrays(gl.POINTS, 0, 1);
    
    const pixels = new Uint8Array(4);
    gl.readPixels(32, 32, 1, 1, gl.RGBA, gl.UNSIGNED_BYTE, pixels);
    // Expected: (1.0, 0.5, 0.0) * 0.8 = (0.8, 0.4, 0.0) = (204, 102, 0, 255)
    const expected = [204, 102, 0, 255];
    assertPixelsEqual(pixels, expected, 3, 'Multiple varyings test');
  } finally {
    gl.destroy();
  }
});
