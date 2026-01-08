import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';
import { perspective, translate, rotateX, rotateY } from './cubeTestUtils.js';

// Reference colors
const GOLD = { r: 255, g: 215, b: 0 };
const CORNFLOWER_BLUE = { r: 100, g: 149, b: 237 };

function colorDistanceSq(px, col) {
  const dr = px[0] - col.r;
  const dg = px[1] - col.g;
  const db = px[2] - col.b;
  return dr * dr + dg * dg + db * db;
}

function classifyPixels(pixels) {
  // pixels is a Uint8Array of RGBA bytes in GL's readPixels ordering (bottom-up)
  const width = 640;
  const height = 480;
  const total = width * height;
  let transparent = 0;
  let blue = 0;
  let gold = 0;
  let other = 0;

  for (let i = 0; i < pixels.length; i += 4) {
    const r = pixels[i];
    const g = pixels[i + 1];
    const b = pixels[i + 2];
    const a = pixels[i + 3];

    if (a === 0) {
      transparent++;
      continue;
    }

    if (colorDistanceSq([r, g, b], CORNFLOWER_BLUE) <= 30 * 30) {
      blue++;
    } else if (colorDistanceSq([r, g, b], GOLD) <= 30 * 30) {
      gold++;
    } else {
      other++;
    }
  }

  return {
    total,
    transparentPct: (transparent / total) * 100,
    bluePct: (blue / total) * 100,
    goldPct: (gold / total) * 100,
    otherPct: (other / total) * 100,
  };
}

test('Cube regression: rendered pixels match output.png color stats', async () => {
  const gl = await webGL2();
  try {
    gl.viewport(0, 0, 640, 480);

    // Shaders (same as demo)
    const vsSource = `#version 300 es
    layout(location = 0) in vec3 position;
    layout(location = 1) in vec2 uv;
    uniform mat4 u_mvp;
    out vec2 v_uv;
    void main() { v_uv = uv; gl_Position = u_mvp * vec4(position, 1.0); }
    `;

    const fsSource = `#version 300 es
    precision highp float;
    uniform texture2D u_texture;
    uniform sampler u_sampler;
    in vec2 v_uv;
    out vec4 fragColor;
    void main() { fragColor = texture(sampler2D(u_texture, u_sampler), v_uv); }
    `;

    const vs = gl.createShader(gl.VERTEX_SHADER);
    gl.shaderSource(vs, vsSource);
    gl.compileShader(vs);
    assert.strictEqual(gl.getShaderParameter(vs, gl.COMPILE_STATUS), true);

    const fs = gl.createShader(gl.FRAGMENT_SHADER);
    gl.shaderSource(fs, fsSource);
    gl.compileShader(fs);
    assert.strictEqual(gl.getShaderParameter(fs, gl.COMPILE_STATUS), true);

    const program = gl.createProgram();
    gl.attachShader(program, vs);
    gl.attachShader(program, fs);
    gl.linkProgram(program);
    assert.strictEqual(gl.getProgramParameter(program, gl.LINK_STATUS), true);
    gl.useProgram(program);

    // Vertex buffer (same as demo)
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

    // Texture (same 16x16 checkerboard)
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

    const uTextureLoc = gl.getUniformLocation(program, 'u_texture');
    gl.uniform1i(uTextureLoc, 0);
    const uSamplerLoc = gl.getUniformLocation(program, 'u_sampler');
    gl.uniform1i(uSamplerLoc, 0);

    // MVP
    let mvp = perspective(Math.PI / 4, 640 / 480, 0.1, 100.0);
    mvp = translate(mvp, 0, 0, -3);
    mvp = rotateX(mvp, 0.5);
    mvp = rotateY(mvp, 0.8);
    const mvpLoc = gl.getUniformLocation(program, 'u_mvp');
    gl.uniformMatrix4fv(mvpLoc, false, mvp);

    // Render
    gl.clearColor(0.0, 0.0, 0.0, 0.0);
    gl.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT);
    gl.drawArrays(gl.TRIANGLES, 0, 36);

    const pixels = new Uint8Array(640 * 480 * 4);
    gl.readPixels(0, 0, 640, 480, gl.RGBA, gl.UNSIGNED_BYTE, pixels);

    // Reference stats (precomputed from canonical rendering of demo output)
    const REFERENCE_STATS = {
      transparentPct: 78.8955078125,
      bluePct: 10.52734375,
      goldPct: 10.5771484375
    };

    const outStats = classifyPixels(pixels);

    // Allow small absolute difference in percentage points
    const EPS = 2.0; // percent

    assert.ok(Math.abs(REFERENCE_STATS.transparentPct - outStats.transparentPct) <= EPS,
      `Transparent percentage differs: ref=${REFERENCE_STATS.transparentPct.toFixed(2)}% got=${outStats.transparentPct.toFixed(2)}%`);

    assert.ok(Math.abs(REFERENCE_STATS.bluePct - outStats.bluePct) <= EPS,
      `Blue percentage differs: ref=${REFERENCE_STATS.bluePct.toFixed(2)}% got=${outStats.bluePct.toFixed(2)}%`);

    assert.ok(Math.abs(REFERENCE_STATS.goldPct - outStats.goldPct) <= EPS,
      `Gold percentage differs: ref=${REFERENCE_STATS.goldPct.toFixed(2)}% got=${outStats.goldPct.toFixed(2)}%`);

  } finally {
    gl.destroy();
  }
});
