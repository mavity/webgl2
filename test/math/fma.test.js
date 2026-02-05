import { test } from 'node:test';
import assert from 'node:assert';
import { webGL2 } from '../../index.js';

test('Math Builtin: fma', async (t) => {
  const gl = await webGL2({ size: { width: 1, height: 1 } });

  const vs = `#version 300 es
        in vec4 position;
        void main() { gl_Position = position; }
    `;

  const fs = `#version 300 es
        precision highp float;
        uniform float u_a;
        uniform float u_b;
        uniform float u_c;
        out vec4 outColor;
        void main() {
            float v = fma(u_a, u_b, u_c);
            outColor = vec4(v, 0.0, 0.0, 1.0);
        }
    `;

  const program = gl.createProgram();
  const vShader = gl.createShader(gl.VERTEX_SHADER);
  gl.shaderSource(vShader, vs);
  gl.compileShader(vShader);
  const fShader = gl.createShader(gl.FRAGMENT_SHADER);
  gl.shaderSource(fShader, fs);
  gl.compileShader(fShader);
  gl.attachShader(program, vShader);
  gl.attachShader(program, fShader);
  gl.linkProgram(program);

  if (!gl.getProgramParameter(program, gl.LINK_STATUS)) {
    throw new Error(gl.getProgramInfoLog(program));
  }

  gl.useProgram(program);
  const uALoc = gl.getUniformLocation(program, 'u_a');
  const uBLoc = gl.getUniformLocation(program, 'u_b');
  const uCLoc = gl.getUniformLocation(program, 'u_c');

  // Create a 1x1 floating-point framebuffer to avoid 8-bit quantization/clamping.
  const tex = gl.createTexture();
  gl.bindTexture(gl.TEXTURE_2D, tex);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.NEAREST);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.NEAREST);
  gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA32F, 1, 1, 0, gl.RGBA, gl.FLOAT, null);
  const fb = gl.createFramebuffer();
  gl.bindFramebuffer(gl.FRAMEBUFFER, fb);
  gl.framebufferTexture2D(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.TEXTURE_2D, tex, 0);
  if (gl.checkFramebufferStatus(gl.FRAMEBUFFER) !== gl.FRAMEBUFFER_COMPLETE) {
    throw new Error('Float framebuffer not supported');
  }
  gl.viewport(0, 0, 1, 1);

  const as = [1.0, 2.5, -1.0];
  const bs = [0.0, -1.5, 3.0];
  const cs = [0.0, 1.0, -2.0];

  for (const a of as) {
    for (const b of bs) {
      for (const c of cs) {
        gl.uniform1f(uALoc, a);
        gl.uniform1f(uBLoc, b);
        gl.uniform1f(uCLoc, c);
        gl.drawArrays(gl.POINTS, 0, 1);
        const pixels = new Float32Array(4);
        gl.readPixels(0, 0, 1, 1, gl.RGBA, gl.FLOAT, pixels);
        const result = pixels[0];
        const expected = a * b + c;
        assert(Math.abs(result - expected) < 1e-6, `fma(${a}, ${b}, ${c}) failed (got ${result}, expected ${expected})`);
      }
    }
  }
  gl.bindFramebuffer(gl.FRAMEBUFFER, null);
  gl.deleteFramebuffer(fb);
  gl.deleteTexture(tex);
});


test('Math Builtin: fma vector', async (t) => {
  const gl = await webGL2({ size: { width: 1, height: 1 } });

  const vs = `#version 300 es
        in vec4 position;
        void main() { gl_Position = position; }
    `;

  const fs = `#version 300 es
        precision highp float;
        uniform vec2 u_a;
        uniform vec2 u_b;
        uniform vec2 u_c;
        out vec4 outColor;
        void main() {
            vec2 r = fma(u_a, u_b, u_c);
            outColor = vec4(r.x, 0.0, 0.0, 1.0);
        }
    `;

  const program = gl.createProgram();
  const vShader = gl.createShader(gl.VERTEX_SHADER);
  gl.shaderSource(vShader, vs);
  gl.compileShader(vShader);
  const fShader = gl.createShader(gl.FRAGMENT_SHADER);
  gl.shaderSource(fShader, fs);
  gl.compileShader(fShader);
  gl.attachShader(program, vShader);
  gl.attachShader(program, fShader);
  gl.linkProgram(program);

  if (!gl.getProgramParameter(program, gl.LINK_STATUS)) {
    throw new Error(gl.getProgramInfoLog(program));
  }

  gl.useProgram(program);
  gl.uniform2f(gl.getUniformLocation(program, 'u_a'), 1.5, -2.0);
  gl.uniform2f(gl.getUniformLocation(program, 'u_b'), 2.0, 0.25);
  gl.uniform2f(gl.getUniformLocation(program, 'u_c'), -1.0, 0.5);

  // Create a 1x1 floating-point framebuffer
  const tex = gl.createTexture();
  gl.bindTexture(gl.TEXTURE_2D, tex);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.NEAREST);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.NEAREST);
  gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA32F, 1, 1, 0, gl.RGBA, gl.FLOAT, null);
  const fb = gl.createFramebuffer();
  gl.bindFramebuffer(gl.FRAMEBUFFER, fb);
  gl.framebufferTexture2D(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.TEXTURE_2D, tex, 0);
  if (gl.checkFramebufferStatus(gl.FRAMEBUFFER) !== gl.FRAMEBUFFER_COMPLETE) {
    throw new Error('Float framebuffer not supported');
  }
  gl.viewport(0, 0, 1, 1);

  gl.drawArrays(gl.POINTS, 0, 1);
  const pixels = new Float32Array(4);
  gl.readPixels(0, 0, 1, 1, gl.RGBA, gl.FLOAT, pixels);
  const result = pixels[0];
  const expected = 1.5 * 2.0 + (-1.0);
  assert(Math.abs(result - expected) < 1e-6, `fma(vec2).x failed (got ${result}, expected ${expected})`);
  gl.bindFramebuffer(gl.FRAMEBUFFER, null);
  gl.deleteFramebuffer(fb);
  gl.deleteTexture(tex);
});