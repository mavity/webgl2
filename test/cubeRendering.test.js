import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';
import { perspective, multiply, translate, rotateX, rotateY, identity } from './cubeTestUtils.js';

// Test color constants (RGB values)
const COLORS = {
  GOLD: { r: 255, g: 215, b: 0 },
  CORNFLOWER_BLUE: { r: 100, g: 149, b: 237 },
  ORANGE: { r: 255, g: 165, b: 0 },
  PURPLE: { r: 128, g: 0, b: 128 },
  CYAN: { r: 0, g: 255, b: 255 },
  MAGENTA: { r: 255, g: 0, b: 255 },
};

// ============================================================================
// Phase 1: Shader Compilation & Program Setup
// ============================================================================

test('Cube Test 1: Vertex shader compilation with MVP matrix uniform', async () => {
  const gl = await webGL2();
  try {
    const vsSource = `#version 300 es
    layout(location = 0) in vec3 position;
    layout(location = 1) in vec2 uv;
    
    uniform mat4 u_mvp;
    
    out vec2 v_uv;
    
    void main() {
        v_uv = uv;
        gl_Position = u_mvp * vec4(position, 1.0);
    }
    `;

    const vs = gl.createShader(gl.VERTEX_SHADER);
    assert.ok(vs, 'Vertex shader should be created');

    gl.shaderSource(vs, vsSource);
    gl.compileShader(vs);

    const compileStatus = gl.getShaderParameter(vs, gl.COMPILE_STATUS);
    assert.strictEqual(compileStatus, true, 'Vertex shader should compile successfully');

    // Note: Even successfully compiled shaders may have info logs (warnings, etc.)
    // So we don't assert log length is 0, we just check compilation status
  } finally {
    gl.destroy();
  }
});

test('Cube Test 2: Fragment shader compilation with texture and sampler uniforms', async () => {
  const gl = await webGL2();
  try {
    const fsSource = `#version 300 es
    precision highp float;
    uniform texture2D u_texture;
    uniform sampler u_sampler;
    in vec2 v_uv;
    out vec4 fragColor;
    
    void main() {
        fragColor = texture(sampler2D(u_texture, u_sampler), v_uv);
    }
    `;

    const fs = gl.createShader(gl.FRAGMENT_SHADER);
    assert.ok(fs, 'Fragment shader should be created');

    gl.shaderSource(fs, fsSource);
    gl.compileShader(fs);

    const compileStatus = gl.getShaderParameter(fs, gl.COMPILE_STATUS);
    assert.strictEqual(compileStatus, true, 'Fragment shader should compile successfully');

    // Note: Even successfully compiled shaders may have info logs (warnings, etc.)
    // So we don't assert log length is 0, we just check compilation status
  } finally {
    gl.destroy();
  }
});

test('Cube Test 3: Program linking with vertex and fragment shaders', async () => {
  const gl = await webGL2();
  try {
    const vsSource = `#version 300 es
    layout(location = 0) in vec3 position;
    layout(location = 1) in vec2 uv;
    uniform mat4 u_mvp;
    out vec2 v_uv;
    void main() {
        v_uv = uv;
        gl_Position = u_mvp * vec4(position, 1.0);
    }
    `;

    const fsSource = `#version 300 es
    precision highp float;
    uniform texture2D u_texture;
    uniform sampler u_sampler;
    in vec2 v_uv;
    out vec4 fragColor;
    void main() {
        fragColor = texture(sampler2D(u_texture, u_sampler), v_uv);
    }
    `;

    const vs = gl.createShader(gl.VERTEX_SHADER);
    gl.shaderSource(vs, vsSource);
    gl.compileShader(vs);

    const fs = gl.createShader(gl.FRAGMENT_SHADER);
    gl.shaderSource(fs, fsSource);
    gl.compileShader(fs);

    const program = gl.createProgram();
    assert.ok(program, 'Program should be created');

    gl.attachShader(program, vs);
    gl.attachShader(program, fs);
    gl.linkProgram(program);

    const linkStatus = gl.getProgramParameter(program, gl.LINK_STATUS);
    assert.strictEqual(linkStatus, true, 'Program should link successfully');
  } finally {
    gl.destroy();
  }
});

test('Cube Test 4: Program attribute locations (position and UV)', async () => {
  const gl = await webGL2();
  try {
    const vsSource = `#version 300 es
    layout(location = 0) in vec3 position;
    layout(location = 1) in vec2 uv;
    uniform mat4 u_mvp;
    out vec2 v_uv;
    void main() {
        v_uv = uv;
        gl_Position = u_mvp * vec4(position, 1.0);
    }
    `;

    const fsSource = `#version 300 es
    precision highp float;
    uniform texture2D u_texture;
    uniform sampler u_sampler;
    in vec2 v_uv;
    out vec4 fragColor;
    void main() {
        fragColor = texture(sampler2D(u_texture, u_sampler), v_uv);
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

    // Check attribute locations
    const positionLoc = gl.getAttribLocation(program, 'position');
    assert.strictEqual(positionLoc, 0, 'Position attribute should be at location 0');

    const uvLoc = gl.getAttribLocation(program, 'uv');
    assert.strictEqual(uvLoc, 1, 'UV attribute should be at location 1');
  } finally {
    gl.destroy();
  }
});

// ============================================================================
// Phase 2: Vertex Buffer & Geometry
// ============================================================================

test('Cube Test 5: Cube vertex buffer creation and data upload', async () => {
  const gl = await webGL2();
  try {
    // Cube data: 6 faces × 6 vertices per face (2 triangles) × 5 floats per vertex (x,y,z,u,v)
    const vertices = new Float32Array([
      // Front face
      -0.5, -0.5, 0.5, 0.0, 0.0,
      0.5, -0.5, 0.5, 1.0, 0.0,
      0.5, 0.5, 0.5, 1.0, 1.0,
      -0.5, -0.5, 0.5, 0.0, 0.0,
      0.5, 0.5, 0.5, 1.0, 1.0,
      -0.5, 0.5, 0.5, 0.0, 1.0,

      // Back face
      -0.5, -0.5, -0.5, 0.0, 0.0,
      -0.5, 0.5, -0.5, 0.0, 1.0,
      0.5, 0.5, -0.5, 1.0, 1.0,
      -0.5, -0.5, -0.5, 0.0, 0.0,
      0.5, 0.5, -0.5, 1.0, 1.0,
      0.5, -0.5, -0.5, 1.0, 0.0,

      // Top face
      -0.5, 0.5, -0.5, 0.0, 0.0,
      -0.5, 0.5, 0.5, 0.0, 1.0,
      0.5, 0.5, 0.5, 1.0, 1.0,
      -0.5, 0.5, -0.5, 0.0, 0.0,
      0.5, 0.5, 0.5, 1.0, 1.0,
      0.5, 0.5, -0.5, 1.0, 0.0,

      // Bottom face
      -0.5, -0.5, -0.5, 0.0, 0.0,
      0.5, -0.5, -0.5, 1.0, 0.0,
      0.5, -0.5, 0.5, 1.0, 1.0,
      -0.5, -0.5, -0.5, 0.0, 0.0,
      0.5, -0.5, 0.5, 1.0, 1.0,
      -0.5, -0.5, 0.5, 0.0, 1.0,

      // Right face
      0.5, -0.5, -0.5, 0.0, 0.0,
      0.5, 0.5, -0.5, 0.0, 1.0,
      0.5, 0.5, 0.5, 1.0, 1.0,
      0.5, -0.5, -0.5, 0.0, 0.0,
      0.5, 0.5, 0.5, 1.0, 1.0,
      0.5, -0.5, 0.5, 1.0, 0.0,

      // Left face
      -0.5, -0.5, -0.5, 0.0, 0.0,
      -0.5, -0.5, 0.5, 1.0, 0.0,
      -0.5, 0.5, 0.5, 1.0, 1.0,
      -0.5, -0.5, -0.5, 0.0, 0.0,
      -0.5, 0.5, 0.5, 1.0, 1.0,
      -0.5, 0.5, -0.5, 0.0, 1.0,
    ]);

    const buffer = gl.createBuffer();
    assert.ok(buffer, 'Buffer should be created');

    gl.bindBuffer(gl.ARRAY_BUFFER, buffer);
    gl.bufferData(gl.ARRAY_BUFFER, vertices, gl.STATIC_DRAW);

    // Verify buffer was created and data was uploaded
    const bufferSize = gl.getBufferParameter(gl.ARRAY_BUFFER, gl.BUFFER_SIZE);
    assert.strictEqual(bufferSize, vertices.byteLength, 'Buffer size should match vertex data size');
    assert.strictEqual(vertices.length, 180, 'Should have 180 floats (36 vertices × 5 floats)');
  } finally {
    gl.destroy();
  }
});

test('Cube Test 6: Vertex attribute pointer setup (position at location 0, UV at location 1)', async () => {
  const gl = await webGL2();
  try {
    const vertices = new Float32Array([
      -0.5, -0.5, 0.5, 0.0, 0.0,
      0.5, -0.5, 0.5, 1.0, 0.0,
      0.5, 0.5, 0.5, 1.0, 1.0,
    ]);

    const buffer = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, buffer);
    gl.bufferData(gl.ARRAY_BUFFER, vertices, gl.STATIC_DRAW);

    // Set up attribute pointers: 3 floats for position, 2 for UV, stride is 20 bytes
    gl.enableVertexAttribArray(0);
    gl.vertexAttribPointer(0, 3, gl.FLOAT, false, 20, 0);

    gl.enableVertexAttribArray(1);
    gl.vertexAttribPointer(1, 2, gl.FLOAT, false, 20, 12);

    // Verify attributes are enabled
    const enabled0 = gl.getVertexAttrib(0, gl.VERTEX_ATTRIB_ARRAY_ENABLED);
    assert.strictEqual(enabled0, true, 'Position attribute should be enabled');

    const enabled1 = gl.getVertexAttrib(1, gl.VERTEX_ATTRIB_ARRAY_ENABLED);
    assert.strictEqual(enabled1, true, 'UV attribute should be enabled');
  } finally {
    gl.destroy();
  }
});

test('Cube Test 7: Vertex data validation (36 vertices, 6 faces)', async () => {
  const gl = await webGL2();
  try {
    const vertices = new Float32Array([
      // Front face (6 vertices)
      -0.5, -0.5, 0.5, 0.0, 0.0,
      0.5, -0.5, 0.5, 1.0, 0.0,
      0.5, 0.5, 0.5, 1.0, 1.0,
      -0.5, -0.5, 0.5, 0.0, 0.0,
      0.5, 0.5, 0.5, 1.0, 1.0,
      -0.5, 0.5, 0.5, 0.0, 1.0,

      // Back face (6 vertices)
      -0.5, -0.5, -0.5, 0.0, 0.0,
      -0.5, 0.5, -0.5, 0.0, 1.0,
      0.5, 0.5, -0.5, 1.0, 1.0,
      -0.5, -0.5, -0.5, 0.0, 0.0,
      0.5, 0.5, -0.5, 1.0, 1.0,
      0.5, -0.5, -0.5, 1.0, 0.0,

      // Top face (6 vertices)
      -0.5, 0.5, -0.5, 0.0, 0.0,
      -0.5, 0.5, 0.5, 0.0, 1.0,
      0.5, 0.5, 0.5, 1.0, 1.0,
      -0.5, 0.5, -0.5, 0.0, 0.0,
      0.5, 0.5, 0.5, 1.0, 1.0,
      0.5, 0.5, -0.5, 1.0, 0.0,

      // Bottom face (6 vertices)
      -0.5, -0.5, -0.5, 0.0, 0.0,
      0.5, -0.5, -0.5, 1.0, 0.0,
      0.5, -0.5, 0.5, 1.0, 1.0,
      -0.5, -0.5, -0.5, 0.0, 0.0,
      0.5, -0.5, 0.5, 1.0, 1.0,
      -0.5, -0.5, 0.5, 0.0, 1.0,

      // Right face (6 vertices)
      0.5, -0.5, -0.5, 0.0, 0.0,
      0.5, 0.5, -0.5, 0.0, 1.0,
      0.5, 0.5, 0.5, 1.0, 1.0,
      0.5, -0.5, -0.5, 0.0, 0.0,
      0.5, 0.5, 0.5, 1.0, 1.0,
      0.5, -0.5, 0.5, 1.0, 0.0,

      // Left face (6 vertices)
      -0.5, -0.5, -0.5, 0.0, 0.0,
      -0.5, -0.5, 0.5, 1.0, 0.0,
      -0.5, 0.5, 0.5, 1.0, 1.0,
      -0.5, -0.5, -0.5, 0.0, 0.0,
      -0.5, 0.5, 0.5, 1.0, 1.0,
      -0.5, 0.5, -0.5, 0.0, 1.0,
    ]);

    // Validate vertex count
    const vertexCount = vertices.length / 5; // 5 floats per vertex
    assert.strictEqual(vertexCount, 36, 'Should have exactly 36 vertices');

    // Validate face count (6 vertices per face)
    const faceCount = vertexCount / 6;
    assert.strictEqual(faceCount, 6, 'Should have exactly 6 faces');

    // Verify all position coordinates are within expected cube bounds [-0.5, 0.5]
    for (let i = 0; i < vertices.length; i += 5) {
      const x = vertices[i];
      const y = vertices[i + 1];
      const z = vertices[i + 2];
      assert.ok(x >= -0.5 && x <= 0.5, `X coordinate ${x} should be in range [-0.5, 0.5]`);
      assert.ok(y >= -0.5 && y <= 0.5, `Y coordinate ${y} should be in range [-0.5, 0.5]`);
      assert.ok(z >= -0.5 && z <= 0.5, `Z coordinate ${z} should be in range [-0.5, 0.5]`);
    }

    // Verify all UV coordinates are in [0, 1] range
    for (let i = 0; i < vertices.length; i += 5) {
      const u = vertices[i + 3];
      const v = vertices[i + 4];
      assert.ok(u >= 0.0 && u <= 1.0, `U coordinate ${u} should be in range [0.0, 1.0]`);
      assert.ok(v >= 0.0 && v <= 1.0, `V coordinate ${v} should be in range [0.0, 1.0]`);
    }
  } finally {
    gl.destroy();
  }
});

// ============================================================================
// Phase 3: Texture & Sampling
// ============================================================================

test('Cube Test 8: Texture creation with checkerboard pattern (gold and cornflower blue)', async () => {
  const gl = await webGL2();
  try {
    const tex = gl.createTexture();
    assert.ok(tex, 'Texture should be created');

    gl.bindTexture(gl.TEXTURE_2D, tex);

    // Create 16x16 checkerboard pattern
    const texData = new Uint8Array(16 * 16 * 4);
    for (let y = 0; y < 16; y++) {
      for (let x = 0; x < 16; x++) {
        const idx = (y * 16 + x) * 4;
        const isCheck = ((x >> 2) ^ (y >> 2)) & 1;
        if (isCheck) {
          // Gold
          texData[idx] = 255;
          texData[idx + 1] = 215;
          texData[idx + 2] = 0;
          texData[idx + 3] = 255;
        } else {
          // CornflowerBlue
          texData[idx] = 100;
          texData[idx + 1] = 149;
          texData[idx + 2] = 237;
          texData[idx + 3] = 255;
        }
      }
    }

    gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, 16, 16, 0, gl.RGBA, gl.UNSIGNED_BYTE, texData);

    // Verify texture data size is correct
    assert.strictEqual(texData.length, 16 * 16 * 4, 'Texture data should be correct size');
  } finally {
    gl.destroy();
  }
});

test('Cube Test 9: Texture binding and uniform location setup', async () => {
  const gl = await webGL2();
  try {
    const vsSource = `#version 300 es
    layout(location = 0) in vec3 position;
    layout(location = 1) in vec2 uv;
    uniform mat4 u_mvp;
    out vec2 v_uv;
    void main() {
        v_uv = uv;
        gl_Position = u_mvp * vec4(position, 1.0);
    }
    `;

    const fsSource = `#version 300 es
    precision highp float;
    uniform texture2D u_texture;
    uniform sampler u_sampler;
    in vec2 v_uv;
    out vec4 fragColor;
    void main() {
        fragColor = texture(sampler2D(u_texture, u_sampler), v_uv);
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

    // Create and bind texture
    const tex = gl.createTexture();
    gl.bindTexture(gl.TEXTURE_2D, tex);

    // Get uniform locations
    const uTextureLoc = gl.getUniformLocation(program, 'u_texture');
    assert.ok(uTextureLoc !== null, 'u_texture uniform location should be found');

    const uSamplerLoc = gl.getUniformLocation(program, 'u_sampler');
    assert.ok(uSamplerLoc !== null, 'u_sampler uniform location should be found');

    // Set uniform values
    gl.uniform1i(uTextureLoc, 0);
    gl.uniform1i(uSamplerLoc, 0);
  } finally {
    gl.destroy();
  }
});

test('Cube Test 10: Texture data validation (16x16 RGBA pattern)', async () => {
  const gl = await webGL2();
  try {
    const tex = gl.createTexture();
    gl.bindTexture(gl.TEXTURE_2D, tex);

    // Create 16x16 checkerboard pattern
    const texData = new Uint8Array(16 * 16 * 4);
    for (let y = 0; y < 16; y++) {
      for (let x = 0; x < 16; x++) {
        const idx = (y * 16 + x) * 4;
        const isCheck = ((x >> 2) ^ (y >> 2)) & 1;
        if (isCheck) {
          texData[idx] = 255; texData[idx + 1] = 215; texData[idx + 2] = 0; texData[idx + 3] = 255; // Gold
        } else {
          texData[idx] = 100; texData[idx + 1] = 149; texData[idx + 2] = 237; texData[idx + 3] = 255; // CornflowerBlue
        }
      }
    }

    gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, 16, 16, 0, gl.RGBA, gl.UNSIGNED_BYTE, texData);

    // Verify texture dimensions
    assert.strictEqual(texData.length, 16 * 16 * 4, 'Texture data should be 16x16 RGBA');

    // Verify gold color at a known position (e.g., x=0, y=0 should be blue, x=4, y=0 should be gold)
    const blueIdx = (0 * 16 + 0) * 4;
    assert.strictEqual(texData[blueIdx], COLORS.CORNFLOWER_BLUE.r, 'Blue pixel R component');
    assert.strictEqual(texData[blueIdx + 1], COLORS.CORNFLOWER_BLUE.g, 'Blue pixel G component');
    assert.strictEqual(texData[blueIdx + 2], COLORS.CORNFLOWER_BLUE.b, 'Blue pixel B component');

    const goldIdx = (0 * 16 + 4) * 4;
    assert.strictEqual(texData[goldIdx], COLORS.GOLD.r, 'Gold pixel R component');
    assert.strictEqual(texData[goldIdx + 1], COLORS.GOLD.g, 'Gold pixel G component');
    assert.strictEqual(texData[goldIdx + 2], COLORS.GOLD.b, 'Gold pixel B component');
  } finally {
    gl.destroy();
  }
});

// ============================================================================
// Phase 4: Matrix Transformations
// ============================================================================

test('Cube Test 11: Perspective projection matrix calculation', async () => {
  const matrix = perspective(Math.PI / 4, 640 / 480, 0.1, 100.0);

  // Verify matrix has 16 elements
  assert.strictEqual(matrix.length, 16, 'Perspective matrix should have 16 elements');

  // Verify some key values
  const aspect = 640 / 480;
  const f = 1.0 / Math.tan(Math.PI / 8);
  assert.ok(Math.abs(matrix[0] - f / aspect) < 0.0001, 'Matrix[0] should be f/aspect');
  assert.ok(Math.abs(matrix[5] - f) < 0.0001, 'Matrix[5] should be f');
  assert.strictEqual(matrix[11], -1, 'Matrix[11] should be -1');
});

test('Cube Test 12: Translation matrix transformation', async () => {
  const translated = translate(identity, 0, 0, -3);

  // Verify translation is applied
  assert.strictEqual(translated.length, 16, 'Translated matrix should have 16 elements');
  assert.strictEqual(translated[12], 0, 'X translation should be 0');
  assert.strictEqual(translated[13], 0, 'Y translation should be 0');
  assert.strictEqual(translated[14], -3, 'Z translation should be -3');
});

test('Cube Test 13: Rotation matrices (X and Y axis)', async () => {
  const rotatedX = rotateX(identity, 0.5);
  assert.strictEqual(rotatedX.length, 16, 'Rotated X matrix should have 16 elements');

  const rotatedY = rotateY(identity, 0.8);
  assert.strictEqual(rotatedY.length, 16, 'Rotated Y matrix should have 16 elements');

  // Verify rotation affects the correct elements
  const c = Math.cos(0.5);
  const s = Math.sin(0.5);
  assert.ok(Math.abs(rotatedX[5] - c) < 0.0001, 'RotateX should affect [5]');
  assert.ok(Math.abs(rotatedX[6] - s) < 0.0001, 'RotateX should affect [6]');
});

test('Cube Test 14: Combined MVP matrix multiplication', async () => {
  // Calculate full MVP matrix
  let mvp = perspective(Math.PI / 4, 640 / 480, 0.1, 100.0);
  mvp = translate(mvp, 0, 0, -3);
  mvp = rotateX(mvp, 0.5);
  mvp = rotateY(mvp, 0.8);

  assert.strictEqual(mvp.length, 16, 'MVP matrix should have 16 elements');

  // Verify matrix is not identity (transformations were applied)
  let isIdentity = true;
  for (let i = 0; i < 16; i++) {
    if (Math.abs(mvp[i] - identity[i]) > 0.0001) {
      isIdentity = false;
      break;
    }
  }
  assert.strictEqual(isIdentity, false, 'MVP should not be identity matrix');
});

// ============================================================================
// Phase 5: Rendering Pipeline
// ============================================================================

test('Cube Test 15: Clear color and depth buffer', async () => {
  const gl = await webGL2();
  try {
    gl.viewport(0, 0, 640, 480);

    // Clear to black
    gl.clearColor(0.0, 0.0, 0.0, 0.0);
    gl.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT);

    // Read a pixel to verify clear worked
    const pixels = new Uint8Array(4);
    gl.readPixels(320, 240, 1, 1, gl.RGBA, gl.UNSIGNED_BYTE, pixels);

    assert.strictEqual(pixels[0], 0, 'Red should be 0 after clear');
    assert.strictEqual(pixels[1], 0, 'Green should be 0 after clear');
    assert.strictEqual(pixels[2], 0, 'Blue should be 0 after clear');
    assert.strictEqual(pixels[3], 0, 'Alpha should be 0 after clear');
  } finally {
    gl.destroy();
  }
});

test('Cube Test 16: Draw call with 36 vertices (6 faces × 2 triangles × 3 vertices)', async () => {
  const gl = await webGL2();
  try {
    // Setup minimal shader
    const vsSource = `#version 300 es
    layout(location = 0) in vec3 position;
    void main() {
        gl_Position = vec4(position, 1.0);
    }
    `;

    const fsSource = `#version 300 es
    precision highp float;
    out vec4 fragColor;
    void main() {
        fragColor = vec4(1.0, 0.0, 0.0, 1.0);
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

    // Create simple cube vertices (position only)
    const vertices = new Float32Array(36 * 3); // 36 vertices, 3 floats each
    for (let i = 0; i < 36; i++) {
      vertices[i * 3] = 0.0;
      vertices[i * 3 + 1] = 0.0;
      vertices[i * 3 + 2] = 0.0;
    }

    const buffer = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, buffer);
    gl.bufferData(gl.ARRAY_BUFFER, vertices, gl.STATIC_DRAW);

    gl.enableVertexAttribArray(0);
    gl.vertexAttribPointer(0, 3, gl.FLOAT, false, 0, 0);

    gl.clearColor(0, 0, 0, 1);
    gl.clear(gl.COLOR_BUFFER_BIT);

    // Draw 36 vertices as triangles
    gl.drawArrays(gl.TRIANGLES, 0, 36);

    // Verify draw completed without error
    const error = gl.getError();
    assert.strictEqual(error, gl.NO_ERROR, 'Draw call should complete without error');
  } finally {
    gl.destroy();
  }
});

test('Cube Test 17: Pixel readback validation (specific colors at known positions)', async () => {
  const gl = await webGL2();
  try {
    gl.viewport(0, 0, 640, 480);

    // Setup shader that outputs solid color
    const vsSource = `#version 300 es
    layout(location = 0) in vec3 position;
    void main() {
        gl_Position = vec4(position, 1.0);
    }
    `;

    const fsSource = `#version 300 es
    precision highp float;
    out vec4 fragColor;
    void main() {
        fragColor = vec4(1.0, 0.5, 0.25, 1.0);
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

    // Create a simple triangle covering the center
    const vertices = new Float32Array([
      -0.5, -0.5, 0.0,
      0.5, -0.5, 0.0,
      0.0, 0.5, 0.0,
    ]);

    const buffer = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, buffer);
    gl.bufferData(gl.ARRAY_BUFFER, vertices, gl.STATIC_DRAW);

    gl.enableVertexAttribArray(0);
    gl.vertexAttribPointer(0, 3, gl.FLOAT, false, 0, 0);

    gl.clearColor(0, 0, 0, 1);
    gl.clear(gl.COLOR_BUFFER_BIT);

    gl.drawArrays(gl.TRIANGLES, 0, 3);

    // Read pixel at center (should be the triangle color)
    const pixels = new Uint8Array(4);
    gl.readPixels(320, 240, 1, 1, gl.RGBA, gl.UNSIGNED_BYTE, pixels);

    assert.strictEqual(pixels[0], 255, 'Red should be 255');
    assert.strictEqual(pixels[1], 127, 'Green should be 127 (0.5 * 255)');
    assert.strictEqual(pixels[2], 63, 'Blue should be ~63 (0.25 * 255)');
  } finally {
    gl.destroy();
  }
});

test('Cube Test 18: Depth testing API availability', async () => {
  const gl = await webGL2();
  try {
    // Verify depth-related functions exist
    assert.ok(typeof gl.depthFunc === 'function', 'depthFunc should be a function');
    assert.ok(typeof gl.depthMask === 'function', 'depthMask should be a function');
    assert.ok(typeof gl.clearDepth === 'function', 'clearDepth should be a function');

    // Test that depthFunc can be called without error
    // Note: Actual depth testing may not be available in headless mode
    gl.depthFunc(gl.LESS);
  } finally {
    gl.destroy();
  }
});

test('Cube Test 19: Non-primary color verification (gold and blue from texture)', async () => {
  const gl = await webGL2();
  try {
    gl.viewport(0, 0, 64, 64);

    // Create shader that samples texture
    const vsSource = `#version 300 es
    layout(location = 0) in vec2 position;
    layout(location = 1) in vec2 uv;
    out vec2 v_uv;
    void main() {
        v_uv = uv;
        gl_Position = vec4(position, 0.0, 1.0);
    }
    `;

    const fsSource = `#version 300 es
    precision highp float;
    uniform texture2D u_texture;
    uniform sampler u_sampler;
    in vec2 v_uv;
    out vec4 fragColor;
    void main() {
        fragColor = texture(sampler2D(u_texture, u_sampler), v_uv);
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

    // Create texture with specific colors
    const tex = gl.createTexture();
    gl.bindTexture(gl.TEXTURE_2D, tex);

    const texData = new Uint8Array([
      255, 215, 0, 255,   // Gold
      100, 149, 237, 255, // CornflowerBlue
      255, 215, 0, 255,   // Gold
      100, 149, 237, 255, // CornflowerBlue
    ]);
    gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, 2, 2, 0, gl.RGBA, gl.UNSIGNED_BYTE, texData);

    const uTextureLoc = gl.getUniformLocation(program, 'u_texture');
    gl.uniform1i(uTextureLoc, 0);
    const uSamplerLoc = gl.getUniformLocation(program, 'u_sampler');
    gl.uniform1i(uSamplerLoc, 0);

    // Create a quad
    const vertices = new Float32Array([
      -1, -1, 0, 0,
      1, -1, 1, 0,
      1, 1, 1, 1,
      -1, -1, 0, 0,
      1, 1, 1, 1,
      -1, 1, 0, 1,
    ]);

    const buffer = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, buffer);
    gl.bufferData(gl.ARRAY_BUFFER, vertices, gl.STATIC_DRAW);

    gl.enableVertexAttribArray(0);
    gl.vertexAttribPointer(0, 2, gl.FLOAT, false, 16, 0);
    gl.enableVertexAttribArray(1);
    gl.vertexAttribPointer(1, 2, gl.FLOAT, false, 16, 8);

    gl.clearColor(0, 0, 0, 1);
    gl.clear(gl.COLOR_BUFFER_BIT);

    gl.drawArrays(gl.TRIANGLES, 0, 6);

    // Read back and verify gold color
    const pixels = new Uint8Array(4);
    gl.readPixels(16, 16, 1, 1, gl.RGBA, gl.UNSIGNED_BYTE, pixels);

    // Should see one of the texture colors
    const isGold = pixels[0] === 255 && pixels[1] === 215 && pixels[2] === 0;
    const isBlue = pixels[0] === 100 && pixels[1] === 149 && pixels[2] === 237;

    assert.ok(isGold || isBlue, 'Should see either gold or blue color from texture');
  } finally {
    gl.destroy();
  }
});

test('Cube Test 19b: Render and verify specific orange color pixel', async () => {
  const gl = await webGL2();
  try {
    gl.viewport(0, 0, 64, 64);

    // Create shader that outputs a specific orange color (non-primary)
    const vsSource = `#version 300 es
    layout(location = 0) in vec2 position;
    void main() {
        gl_Position = vec4(position, 0.0, 1.0);
    }
    `;

    const fsSource = `#version 300 es
    precision highp float;
    out vec4 fragColor;
    void main() {
        // Orange color
        fragColor = vec4(${COLORS.ORANGE.r / 255}, ${COLORS.ORANGE.g / 255}, 0.0, 1.0);
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

    // Create a fullscreen quad
    const vertices = new Float32Array([
      -1, -1,
      1, -1,
      1, 1,
      -1, -1,
      1, 1,
      -1, 1,
    ]);

    const buffer = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, buffer);
    gl.bufferData(gl.ARRAY_BUFFER, vertices, gl.STATIC_DRAW);

    gl.enableVertexAttribArray(0);
    gl.vertexAttribPointer(0, 2, gl.FLOAT, false, 0, 0);

    gl.clearColor(0, 0, 0, 1);
    gl.clear(gl.COLOR_BUFFER_BIT);

    gl.drawArrays(gl.TRIANGLES, 0, 6);

    // Read back center pixel
    const pixels = new Uint8Array(4);
    gl.readPixels(32, 32, 1, 1, gl.RGBA, gl.UNSIGNED_BYTE, pixels);

    // Verify orange color
    assert.strictEqual(pixels[0], COLORS.ORANGE.r, 'Red should be 255 for orange');
    assert.ok(Math.abs(pixels[1] - COLORS.ORANGE.g) <= 1, 'Green should be ~165 for orange');
    assert.strictEqual(pixels[2], COLORS.ORANGE.b, 'Blue should be 0 for orange');
    assert.strictEqual(pixels[3], 255, 'Alpha should be 255');
  } finally {
    gl.destroy();
  }
});

test('Cube Test 19c: Render and verify purple color (non-primary)', async () => {
  const gl = await webGL2();
  try {
    gl.viewport(0, 0, 64, 64);

    const vsSource = `#version 300 es
    layout(location = 0) in vec2 position;
    void main() {
        gl_Position = vec4(position, 0.0, 1.0);
    }
    `;

    const fsSource = `#version 300 es
    precision highp float;
    out vec4 fragColor;
    void main() {
        // Purple color
        fragColor = vec4(${COLORS.PURPLE.r / 255}, 0.0, ${COLORS.PURPLE.b / 255}, 1.0);
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

    const vertices = new Float32Array([
      -1, -1, 1, -1, 1, 1,
      -1, -1, 1, 1, -1, 1,
    ]);

    const buffer = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, buffer);
    gl.bufferData(gl.ARRAY_BUFFER, vertices, gl.STATIC_DRAW);

    gl.enableVertexAttribArray(0);
    gl.vertexAttribPointer(0, 2, gl.FLOAT, false, 0, 0);

    gl.clearColor(0, 0, 0, 1);
    gl.clear(gl.COLOR_BUFFER_BIT);
    gl.drawArrays(gl.TRIANGLES, 0, 6);

    const pixels = new Uint8Array(4);
    gl.readPixels(32, 32, 1, 1, gl.RGBA, gl.UNSIGNED_BYTE, pixels);

    // Verify purple color
    assert.ok(Math.abs(pixels[0] - COLORS.PURPLE.r) <= 1, 'Red should be ~128 for purple');
    assert.strictEqual(pixels[1], COLORS.PURPLE.g, 'Green should be 0 for purple');
    assert.ok(Math.abs(pixels[2] - COLORS.PURPLE.b) <= 1, 'Blue should be ~128 for purple');
  } finally {
    gl.destroy();
  }
});

test('Cube Test 19d: Render and verify cyan color (non-primary)', async () => {
  const gl = await webGL2();
  try {
    gl.viewport(0, 0, 64, 64);

    const vsSource = `#version 300 es
    layout(location = 0) in vec2 position;
    void main() {
        gl_Position = vec4(position, 0.0, 1.0);
    }
    `;

    const fsSource = `#version 300 es
    precision highp float;
    out vec4 fragColor;
    void main() {
        // Cyan color
        fragColor = vec4(0.0, 1.0, 1.0, 1.0);
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

    const vertices = new Float32Array([
      -1, -1, 1, -1, 1, 1,
      -1, -1, 1, 1, -1, 1,
    ]);

    const buffer = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, buffer);
    gl.bufferData(gl.ARRAY_BUFFER, vertices, gl.STATIC_DRAW);

    gl.enableVertexAttribArray(0);
    gl.vertexAttribPointer(0, 2, gl.FLOAT, false, 0, 0);

    gl.clearColor(0, 0, 0, 1);
    gl.clear(gl.COLOR_BUFFER_BIT);
    gl.drawArrays(gl.TRIANGLES, 0, 6);

    const pixels = new Uint8Array(4);
    gl.readPixels(32, 32, 1, 1, gl.RGBA, gl.UNSIGNED_BYTE, pixels);

    // Verify cyan color
    assert.strictEqual(pixels[0], COLORS.CYAN.r, 'Red should be 0 for cyan');
    assert.strictEqual(pixels[1], COLORS.CYAN.g, 'Green should be 255 for cyan');
    assert.strictEqual(pixels[2], COLORS.CYAN.b, 'Blue should be 255 for cyan');
  } finally {
    gl.destroy();
  }
});

test('Cube Test 19e: Render and verify magenta/pink color (non-primary)', async () => {
  const gl = await webGL2();
  try {
    gl.viewport(0, 0, 64, 64);

    const vsSource = `#version 300 es
    layout(location = 0) in vec2 position;
    void main() {
        gl_Position = vec4(position, 0.0, 1.0);
    }
    `;

    const fsSource = `#version 300 es
    precision highp float;
    out vec4 fragColor;
    void main() {
        // Magenta/Pink color
        fragColor = vec4(1.0, 0.0, 1.0, 1.0);
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

    const vertices = new Float32Array([
      -1, -1, 1, -1, 1, 1,
      -1, -1, 1, 1, -1, 1,
    ]);

    const buffer = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, buffer);
    gl.bufferData(gl.ARRAY_BUFFER, vertices, gl.STATIC_DRAW);

    gl.enableVertexAttribArray(0);
    gl.vertexAttribPointer(0, 2, gl.FLOAT, false, 0, 0);

    gl.clearColor(0, 0, 0, 1);
    gl.clear(gl.COLOR_BUFFER_BIT);
    gl.drawArrays(gl.TRIANGLES, 0, 6);

    const pixels = new Uint8Array(4);
    gl.readPixels(32, 32, 1, 1, gl.RGBA, gl.UNSIGNED_BYTE, pixels);

    // Verify magenta color
    assert.strictEqual(pixels[0], COLORS.MAGENTA.r, 'Red should be 255 for magenta');
    assert.strictEqual(pixels[1], COLORS.MAGENTA.g, 'Green should be 0 for magenta');
    assert.strictEqual(pixels[2], COLORS.MAGENTA.b, 'Blue should be 255 for magenta');
  } finally {
    gl.destroy();
  }
});

test('Cube Test 19f: Render gradient with interpolated non-primary colors', async () => {
  const gl = await webGL2();
  try {
    gl.viewport(0, 0, 64, 64);

    // Vertex shader that passes color through
    const vsSource = `#version 300 es
    layout(location = 0) in vec2 position;
    layout(location = 1) in vec3 color;
    out vec3 v_color;
    void main() {
        v_color = color;
        gl_Position = vec4(position, 0.0, 1.0);
    }
    `;

    const fsSource = `#version 300 es
    precision highp float;
    in vec3 v_color;
    out vec4 fragColor;
    void main() {
        fragColor = vec4(v_color, 1.0);
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

    // Create triangle with different colors at each vertex
    const vertices = new Float32Array([
      // position (x,y), color (r,g,b)
      -0.5, -0.5, 1.0, 0.5, 0.0,  // Orange-red at bottom left
      0.5, -0.5, 0.0, 1.0, 0.5,   // Green-cyan at bottom right
      0.0, 0.5, 0.5, 0.0, 1.0,    // Blue-purple at top
    ]);

    const buffer = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, buffer);
    gl.bufferData(gl.ARRAY_BUFFER, vertices, gl.STATIC_DRAW);

    gl.enableVertexAttribArray(0);
    gl.vertexAttribPointer(0, 2, gl.FLOAT, false, 20, 0);
    gl.enableVertexAttribArray(1);
    gl.vertexAttribPointer(1, 3, gl.FLOAT, false, 20, 8);

    gl.clearColor(0, 0, 0, 1);
    gl.clear(gl.COLOR_BUFFER_BIT);
    gl.drawArrays(gl.TRIANGLES, 0, 3);

    // Read center pixel - should be interpolated color (non-primary)
    const pixels = new Uint8Array(4);
    gl.readPixels(32, 32, 1, 1, gl.RGBA, gl.UNSIGNED_BYTE, pixels);

    // Center should be a mix of all three vertex colors (non-primary)
    // All channels should have some value (not pure red, green, or blue)
    const hasRed = pixels[0] > 50;
    const hasGreen = pixels[1] > 50;
    const hasBlue = pixels[2] > 50;

    assert.ok(hasRed && hasGreen && hasBlue,
      'Center pixel should be interpolated non-primary color with R, G, and B components');
  } finally {
    gl.destroy();
  }
});

// ============================================================================
// Phase 6: Animation & Rotation
// ============================================================================

test('Cube Test 20: Time-based rotation angle calculation', async () => {
  // Test rotation calculation for different elapsed times

  // At 0ms, rotation should be 0
  let elapsedTime = 0;
  let rotationAngle = (elapsedTime / 5000) * Math.PI * 2;
  assert.strictEqual(rotationAngle, 0, 'At 0ms, rotation should be 0');

  // At 2500ms (2.5 seconds), rotation should be π (half rotation)
  elapsedTime = 2500;
  rotationAngle = (elapsedTime / 5000) * Math.PI * 2;
  assert.ok(Math.abs(rotationAngle - Math.PI) < 0.0001, 'At 2.5s, rotation should be π');

  // At 5000ms (5 seconds), rotation should be 2π (full rotation)
  elapsedTime = 5000;
  rotationAngle = (elapsedTime / 5000) * Math.PI * 2;
  assert.ok(Math.abs(rotationAngle - Math.PI * 2) < 0.0001, 'At 5s, rotation should be 2π');
});

test('Cube Test 21: MVP matrix update with rotation', async () => {
  // Calculate MVP at different rotation angles
  const elapsedTime = 1000; // 1 second
  const rotationAngle = (elapsedTime / 5000) * Math.PI * 2;

  let mvp1 = perspective(Math.PI / 4, 640 / 480, 0.1, 100.0);
  mvp1 = translate(mvp1, 0, 0, -3);
  mvp1 = rotateX(mvp1, 0.5);
  mvp1 = rotateY(mvp1, 0.8);

  let mvp2 = perspective(Math.PI / 4, 640 / 480, 0.1, 100.0);
  mvp2 = translate(mvp2, 0, 0, -3);
  mvp2 = rotateX(mvp2, 0.5);
  mvp2 = rotateY(mvp2, 0.8 + rotationAngle);

  // Matrices should be different due to rotation
  let isDifferent = false;
  for (let i = 0; i < 16; i++) {
    if (Math.abs(mvp1[i] - mvp2[i]) > 0.0001) {
      isDifferent = true;
      break;
    }
  }
  assert.strictEqual(isDifferent, true, 'MVP matrices should differ with rotation');
});

test('Cube Test 22: Multiple frames with different rotation angles', async () => {
  // Generate 5 frames with different rotations
  const frames = [];
  for (let frame = 0; frame < 5; frame++) {
    const elapsedTime = frame * 1000; // Each frame is 1 second apart
    const rotationAngle = (elapsedTime / 5000) * Math.PI * 2;

    let mvp = perspective(Math.PI / 4, 640 / 480, 0.1, 100.0);
    mvp = translate(mvp, 0, 0, -3);
    mvp = rotateX(mvp, 0.5);
    mvp = rotateY(mvp, 0.8 + rotationAngle);

    frames.push(mvp);
  }

  // Verify all frames are unique
  assert.strictEqual(frames.length, 5, 'Should have 5 frames');

  for (let i = 0; i < frames.length - 1; i++) {
    let isDifferent = false;
    for (let j = 0; j < 16; j++) {
      if (Math.abs(frames[i][j] - frames[i + 1][j]) > 0.0001) {
        isDifferent = true;
        break;
      }
    }
    assert.strictEqual(isDifferent, true, `Frame ${i} should differ from frame ${i + 1}`);
  }
});

// ============================================================================
// Phase 6: Feature Reproduction (Demo.js parity)
// ============================================================================

test('Feature: Vec3 Attributes + Texture + Complex Matrix + Large Viewport', async () => {
  const gl = await webGL2();
  try {
    gl.viewport(0, 0, 640, 480);

    const vsSource = `#version 300 es
    layout(location = 0) in vec3 position;
    layout(location = 1) in vec2 uv;
    uniform mat4 u_mvp;
    out vec2 v_uv;
    void main() {
        v_uv = uv;
        gl_Position = u_mvp * vec4(position, 1.0);
    }
    `;

    const fsSource = `#version 300 es
    precision highp float;
    in vec2 v_uv;
    uniform texture2D u_texture;
    uniform sampler u_sampler;
    out vec4 fragColor;
    void main() {
        fragColor = texture(sampler2D(u_texture, u_sampler), v_uv);
    }
    `;

    const vs = gl.createShader(gl.VERTEX_SHADER);
    gl.shaderSource(vs, vsSource);
    gl.compileShader(vs);
    if (!gl.getShaderParameter(vs, gl.COMPILE_STATUS)) {
      throw new Error('VS compile failed: ' + gl.getShaderInfoLog(vs));
    }

    const fs = gl.createShader(gl.FRAGMENT_SHADER);
    gl.shaderSource(fs, fsSource);
    gl.compileShader(fs);
    if (!gl.getShaderParameter(fs, gl.COMPILE_STATUS)) {
      throw new Error('FS compile failed: ' + gl.getShaderInfoLog(fs));
    }

    const program = gl.createProgram();
    gl.attachShader(program, vs);
    gl.attachShader(program, fs);
    gl.linkProgram(program);
    gl.useProgram(program);

    const tex = gl.createTexture();
    gl.bindTexture(gl.TEXTURE_2D, tex);
    const texData = new Uint8Array([
      255, 0, 0, 255, 0, 255, 0, 255,
      0, 0, 255, 255, 255, 255, 255, 255
    ]);
    gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, 2, 2, 0, gl.RGBA, gl.UNSIGNED_BYTE, texData);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.NEAREST);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.NEAREST);

    const vertices = new Float32Array([
      -0.5, -0.5, 0.0, 0.0, 0.0,
      0.5, -0.5, 0.0, 1.0, 0.0,
      0.0, 0.5, 0.0, 0.5, 1.0,
    ]);

    const buffer = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, buffer);
    gl.bufferData(gl.ARRAY_BUFFER, vertices, gl.STATIC_DRAW);

    gl.enableVertexAttribArray(0);
    gl.vertexAttribPointer(0, 3, gl.FLOAT, false, 20, 0);

    gl.enableVertexAttribArray(1);
    gl.vertexAttribPointer(1, 2, gl.FLOAT, false, 20, 12);

    // Matrix functions from demo.js
    function perspective(fovy, aspect, near, far) {
      const f = 1.0 / Math.tan(fovy / 2);
      const nf = 1 / (near - far);
      return [
        f / aspect, 0, 0, 0,
        0, f, 0, 0,
        0, 0, (far + near) * nf, -1,
        0, 0, (2 * far * near) * nf, 0
      ];
    }

    function multiply(a, b) {
      const out = new Float32Array(16);
      for (let col = 0; col < 4; col++) {
        for (let row = 0; row < 4; row++) {
          let sum = 0;
          for (let k = 0; k < 4; k++) {
            sum += a[k * 4 + row] * b[col * 4 + k];
          }
          out[col * 4 + row] = sum;
        }
      }
      return out;
    }

    function translate(m, x, y, z) {
      const t = [
        1, 0, 0, 0,
        0, 1, 0, 0,
        0, 0, 1, 0,
        x, y, z, 1
      ];
      return multiply(m, t);
    }

    // Setup Matrix
    // Aspect 640/480
    let mvp = perspective(Math.PI / 4, 640 / 480, 0.1, 100.0);
    mvp = translate(mvp, 0, 0, -3);

    const u_mvp = gl.getUniformLocation(program, 'u_mvp');
    gl.uniformMatrix4fv(u_mvp, false, new Float32Array(mvp));

    const u_texture = gl.getUniformLocation(program, 'u_texture');
    gl.uniform1i(u_texture, 0);
    const u_sampler = gl.getUniformLocation(program, 'u_sampler');
    gl.uniform1i(u_sampler, 0);

    gl.clearColor(0.2, 0.2, 0.2, 1);
    gl.clear(gl.COLOR_BUFFER_BIT);

    gl.drawArrays(gl.TRIANGLES, 0, 3);

    // Read pixel at center (320, 240)
    const pixels = new Uint8Array(4);
    gl.readPixels(320, 240, 1, 1, gl.RGBA, gl.UNSIGNED_BYTE, pixels);

    // Should be textured (not background)
    assert.ok(pixels[0] !== 51 || pixels[1] !== 51 || pixels[2] !== 51, 'Should draw something (not background color)');

  } finally {
    gl.destroy();
  }
});

test('Feature: Shader Function Calls (mimicking demo.js)', async () => {
  const gl = await webGL2();
  try {
    gl.viewport(0, 0, 64, 64);

    const vsSource = `#version 300 es
    layout(location = 0) in vec3 position;
    void main() {
        gl_Position = vec4(position, 1.0);
    }
    `;

    // Fragment shader with function calls, exactly like demo.js
    const fsSource = `#version 300 es
    precision highp float;
    out vec4 fragColor;

    void small_fn_before(float val_noop) {
        val_noop = 1.0;
    }

    void small_fn_after(float val_noop) {
        val_noop = 2.0;
    }

    void main() {
        small_fn_before(3.0);
        // small_fn_after(4.0); // Commented out in demo.js too
        fragColor = vec4(0.0, 1.0, 0.0, 1.0); // Green
    }
    `;

    const vs = gl.createShader(gl.VERTEX_SHADER);
    gl.shaderSource(vs, vsSource);
    gl.compileShader(vs);
    if (!gl.getShaderParameter(vs, gl.COMPILE_STATUS)) {
      throw new Error('VS compile failed: ' + gl.getShaderInfoLog(vs));
    }

    const fs = gl.createShader(gl.FRAGMENT_SHADER);
    gl.shaderSource(fs, fsSource);
    gl.compileShader(fs);
    if (!gl.getShaderParameter(fs, gl.COMPILE_STATUS)) {
      throw new Error('FS compile failed: ' + gl.getShaderInfoLog(fs));
    }

    const program = gl.createProgram();
    gl.attachShader(program, vs);
    gl.attachShader(program, fs);
    gl.linkProgram(program);
    gl.useProgram(program);

    const vertices = new Float32Array([
      -0.5, -0.5, 0.0,
      0.5, -0.5, 0.0,
      0.0, 0.5, 0.0,
    ]);

    const buffer = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, buffer);
    gl.bufferData(gl.ARRAY_BUFFER, vertices, gl.STATIC_DRAW);

    gl.enableVertexAttribArray(0);
    gl.vertexAttribPointer(0, 3, gl.FLOAT, false, 0, 0);

    gl.clearColor(0.2, 0.0, 0.0, 1);
    gl.clear(gl.COLOR_BUFFER_BIT);

    gl.drawArrays(gl.TRIANGLES, 0, 3);

    const pixels = new Uint8Array(4);
    gl.readPixels(32, 32, 1, 1, gl.RGBA, gl.UNSIGNED_BYTE, pixels);

    // Should be Green (0, 255, 0, 255)
    assert.strictEqual(pixels[0], 0, 'Red should be 0');
    assert.strictEqual(pixels[1], 255, 'Green should be 255');
    assert.strictEqual(pixels[2], 0, 'Blue should be 0');

  } finally {
    gl.destroy();
  }
});

test('Feature: Full Demo Reproduction', async () => {
  const gl = await webGL2();
  try {
    gl.viewport(0, 0, 640, 480);

    // Shaders from demo.js
    const vsSource = `#version 300 es
    layout(location = 0) in vec3 position;
    layout(location = 1) in vec2 uv;
    
    uniform mat4 u_mvp;
    
    out vec2 v_uv;
    
    void main() {
        v_uv = uv;
        gl_Position = u_mvp * vec4(position, 1.0);
    }
    `;

    const fsSource = `#version 300 es
    precision highp float;
    uniform texture2D u_texture;
    uniform sampler u_sampler;
    in vec2 v_uv;
    out vec4 fragColor;

    void small_fn_before(float val_noop) {
        val_noop = 1.0;
    }

    void small_fn_after(float val_noop) {
        val_noop = 2.0;
    }

    void main() {
        small_fn_before(3.0);
        // uncomment and it blows
        // small_fn_after(4.0);
        fragColor = texture(sampler2D(u_texture, u_sampler), v_uv);
        // fragColor = vec4(v_uv, 0.0, 1.0);
    }`;

    const vs = gl.createShader(gl.VERTEX_SHADER);
    gl.shaderSource(vs, vsSource);
    gl.compileShader(vs);
    if (!gl.getShaderParameter(vs, gl.COMPILE_STATUS)) {
      throw new Error('VS compile failed: ' + gl.getShaderInfoLog(vs));
    }

    const fs = gl.createShader(gl.FRAGMENT_SHADER);
    gl.shaderSource(fs, fsSource);
    gl.compileShader(fs);
    if (!gl.getShaderParameter(fs, gl.COMPILE_STATUS)) {
      throw new Error('FS compile failed: ' + gl.getShaderInfoLog(fs));
    }

    const program = gl.createProgram();
    gl.attachShader(program, vs);
    gl.attachShader(program, fs);
    gl.linkProgram(program);
    gl.useProgram(program);

    // Cube data from demo.js
    const vertices = new Float32Array([
      // Front face
      -0.5, -0.5, 0.5, 0.0, 0.0,
      0.5, -0.5, 0.5, 1.0, 0.0,
      0.5, 0.5, 0.5, 1.0, 1.0,
      -0.5, -0.5, 0.5, 0.0, 0.0,
      0.5, 0.5, 0.5, 1.0, 1.0,
      -0.5, 0.5, 0.5, 0.0, 1.0,

      // Back face
      -0.5, -0.5, -0.5, 0.0, 0.0,
      -0.5, 0.5, -0.5, 0.0, 1.0,
      0.5, 0.5, -0.5, 1.0, 1.0,
      -0.5, -0.5, -0.5, 0.0, 0.0,
      0.5, 0.5, -0.5, 1.0, 1.0,
      0.5, -0.5, -0.5, 1.0, 0.0,

      // Top face
      -0.5, 0.5, -0.5, 0.0, 0.0,
      -0.5, 0.5, 0.5, 0.0, 1.0,
      0.5, 0.5, 0.5, 1.0, 1.0,
      -0.5, 0.5, -0.5, 0.0, 0.0,
      0.5, 0.5, 0.5, 1.0, 1.0,
      0.5, 0.5, -0.5, 1.0, 0.0,

      // Bottom face
      -0.5, -0.5, -0.5, 0.0, 0.0,
      0.5, -0.5, -0.5, 1.0, 0.0,
      0.5, -0.5, 0.5, 1.0, 1.0,
      -0.5, -0.5, -0.5, 0.0, 0.0,
      0.5, -0.5, 0.5, 1.0, 1.0,
      -0.5, -0.5, 0.5, 0.0, 1.0,

      // Right face
      0.5, -0.5, -0.5, 0.0, 0.0,
      0.5, 0.5, -0.5, 0.0, 1.0,
      0.5, 0.5, 0.5, 1.0, 1.0,
      0.5, -0.5, -0.5, 0.0, 0.0,
      0.5, 0.5, 0.5, 1.0, 1.0,
      0.5, -0.5, 0.5, 1.0, 0.0,

      // Left face
      -0.5, -0.5, -0.5, 0.0, 0.0,
      -0.5, -0.5, 0.5, 1.0, 0.0,
      -0.5, 0.5, 0.5, 1.0, 1.0,
      -0.5, -0.5, -0.5, 0.0, 0.0,
      -0.5, 0.5, 0.5, 1.0, 1.0,
      -0.5, 0.5, -0.5, 0.0, 1.0,
    ]);

    const buffer = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, buffer);
    gl.bufferData(gl.ARRAY_BUFFER, vertices, gl.STATIC_DRAW);

    gl.enableVertexAttribArray(0);
    gl.vertexAttribPointer(0, 3, gl.FLOAT, false, 20, 0);
    gl.enableVertexAttribArray(1);
    gl.vertexAttribPointer(1, 2, gl.FLOAT, false, 20, 12);

    // Texture from demo.js
    const tex = gl.createTexture();
    gl.bindTexture(gl.TEXTURE_2D, tex);
    const texData = new Uint8Array(16 * 16 * 4);
    for (let y = 0; y < 16; y++) {
      for (let x = 0; x < 16; x++) {
        const idx = (y * 16 + x) * 4;
        const isCheck = ((x >> 2) ^ (y >> 2)) & 1;
        if (isCheck) {
          texData[idx] = 255; texData[idx + 1] = 215; texData[idx + 2] = 0; texData[idx + 3] = 255; // Gold
        } else {
          texData[idx] = 100; texData[idx + 1] = 149; texData[idx + 2] = 237; texData[idx + 3] = 255; // CornflowerBlue
        }
      }
    }
    gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, 16, 16, 0, gl.RGBA, gl.UNSIGNED_BYTE, texData);

    const uTextureLoc = gl.getUniformLocation(program, "u_texture");

    gl.uniform1i(uTextureLoc, 0);
    const uSamplerLoc = gl.getUniformLocation(program, "u_sampler");
    gl.uniform1i(uSamplerLoc, 0);

    // Matrix functions from demo.js
    function perspective(fovy, aspect, near, far) {
      const f = 1.0 / Math.tan(fovy / 2);
      const nf = 1 / (near - far);
      return [
        f / aspect, 0, 0, 0,
        0, f, 0, 0,
        0, 0, (far + near) * nf, -1,
        0, 0, (2 * far * near) * nf, 0
      ];
    }

    function multiply(a, b) {
      const out = new Float32Array(16);
      for (let col = 0; col < 4; col++) {
        for (let row = 0; row < 4; row++) {
          let sum = 0;
          for (let k = 0; k < 4; k++) {
            sum += a[k * 4 + row] * b[col * 4 + k];
          }
          out[col * 4 + row] = sum;
        }
      }
      return out;
    }

    function rotateY(m, angle) {
      const c = Math.cos(angle);
      const s = Math.sin(angle);
      const r = [
        c, 0, -s, 0,
        0, 1, 0, 0,
        s, 0, c, 0,
        0, 0, 0, 1
      ];
      return multiply(m, r);
    }

    function rotateX(m, angle) {
      const c = Math.cos(angle);
      const s = Math.sin(angle);
      const r = [
        1, 0, 0, 0,
        0, c, s, 0,
        0, -s, c, 0,
        0, 0, 0, 1
      ];
      return multiply(m, r);
    }

    function translate(m, x, y, z) {
      const t = [
        1, 0, 0, 0,
        0, 1, 0, 0,
        0, 0, 1, 0,
        x, y, z, 1
      ];
      return multiply(m, t);
    }

    // Calculate initial MVP matrix
    let mvp = perspective(Math.PI / 4, 640 / 480, 0.1, 100.0);
    mvp = translate(mvp, 0, 0, -3);
    mvp = rotateX(mvp, 0.5);
    mvp = rotateY(mvp, 0.8);

    const mvpLoc = gl.getUniformLocation(program, "u_mvp");
    gl.uniformMatrix4fv(mvpLoc, false, mvp);

    // Render
    gl.clearColor(0.0, 0.0, 0.0, 0.0);
    gl.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT);
    gl.drawArrays(gl.TRIANGLES, 0, 36);

    // Read pixels
    const pixels = new Uint8Array(640 * 480 * 4);
    gl.readPixels(0, 0, 640, 480, gl.RGBA, gl.UNSIGNED_BYTE, pixels);

    // Check if any pixel is not transparent black
    let hasContent = false;
    for (let i = 0; i < pixels.length; i += 4) {
      if (pixels[i + 3] !== 0) { // Check alpha
        hasContent = true;
        break;
      }
    }

    assert.ok(hasContent, 'Should render something (non-transparent pixels)');

  } finally {
    gl.destroy();
  }
});
