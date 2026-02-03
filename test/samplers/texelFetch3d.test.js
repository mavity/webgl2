import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../../index.js';

// texelFetch on 3D textures — verify coordinates and layer selection

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
void main() {
  gl_Position = vec4(0.0, 0.0, 0.0, 1.0);
  gl_PointSize = 1.0;
}
`;

const FS = `#version 300 es
precision highp float;
uniform sampler3D u_tex;
uniform int u_x;
uniform int u_y;
uniform int u_z;
layout(location = 0) out vec4 fragColor;
void main() {
    fragColor = texelFetch(u_tex, ivec3(u_x, u_y, u_z), 0);
}
`;

test('texelFetch 3D tests', async (t) => {
  await t.test('texelFetch returns correct value from specific 3D layer', async () => {
    const gl = await webGL2();
    try {
      const program = await compileProgram(gl, VS, FS);
      gl.useProgram(program);

      const texture = gl.createTexture();
      gl.bindTexture(gl.TEXTURE_3D, texture);

      // 2x2x2 texture
      const width = 2;
      const height = 2;
      const depth = 2;
      const data = new Uint8Array(width * height * depth * 4);

      // (x,y,z) -> red channel gets unique-ish value
      for (let z = 0; z < depth; z++) {
        for (let y = 0; y < height; y++) {
          for (let x = 0; x < width; x++) {
            const idx = (x + y * width + z * width * height) * 4;
            // Use values that fit in 8-bit but are distinct
            data[idx] = (x + y * 2 + z * 4) * 20 + 10; 
            data[idx + 1] = 0;
            data[idx + 2] = 0;
            data[idx + 3] = 255;
          }
        }
      }

      gl.texImage3D(gl.TEXTURE_3D, 0, gl.RGBA8, width, height, depth, 0, gl.RGBA, gl.UNSIGNED_BYTE, data);

      const locX = gl.getUniformLocation(program, 'u_x');
      const locY = gl.getUniformLocation(program, 'u_y');
      const locZ = gl.getUniformLocation(program, 'u_z');
      const texLoc = gl.getUniformLocation(program, 'u_tex');
      gl.uniform1i(texLoc, 0);

      const checkCoord = (x, y, z) => {
        gl.uniform1i(locX, x);
        gl.uniform1i(locY, y);
        gl.uniform1i(locZ, z);
        gl.clearColor(0, 0, 0, 1);
        gl.clear(gl.COLOR_BUFFER_BIT);
        gl.drawArrays(gl.POINTS, 0, 1);

        const pixels = new Uint8Array(4);
        gl.readPixels(320, 240, 1, 1, gl.RGBA, gl.UNSIGNED_BYTE, pixels);

        const expectedR = (x + y * 2 + z * 4) * 20 + 10;
        assert.strictEqual(pixels[0], expectedR, `At (${x},${y},${z}), expected R=${expectedR} but got ${pixels[0]}`);
        assert.strictEqual(pixels[3], 255);
      };

      checkCoord(0, 0, 0);
      checkCoord(1, 0, 0);
      checkCoord(0, 1, 0);
      checkCoord(1, 1, 0);
      checkCoord(0, 0, 1);
      checkCoord(1, 0, 1);
      checkCoord(0, 1, 1);
      checkCoord(1, 1, 1);

    } finally {
      gl.destroy();
    }
  });
});
