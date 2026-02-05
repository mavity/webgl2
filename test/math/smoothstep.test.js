import { test } from 'node:test';
import assert from 'node:assert';
import { webGL2 } from '../../index.js';

function computeSmoothStep(edge0, edge1, x) {
  const t = Math.max(0, Math.min(1, (x - edge0) / (edge1 - edge0)));
  return t * t * (3 - 2 * t);
}

test('Math Builtin: smoothstep', async (t) => {
  const gl = await webGL2({ size: { width: 1, height: 1 } });

  const vs = `#version 300 es
        in vec4 position;
        void main() { gl_Position = position; }
    `;

  const fs = `#version 300 es
        precision highp float;
        uniform float u_e0;
        uniform float u_e1;
        uniform float u_x;
        out vec4 outColor;
        void main() {
            float v = smoothstep(u_e0, u_e1, u_x);
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
  const uE0Loc = gl.getUniformLocation(program, 'u_e0');
  const uE1Loc = gl.getUniformLocation(program, 'u_e1');
  const uXLoc = gl.getUniformLocation(program, 'u_x');

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

  const cases = [
    { e0: 0.0, e1: 1.0, x: -0.5 },
    { e0: 0.0, e1: 1.0, x: 0.25 },
    { e0: 0.0, e1: 1.0, x: 0.75 },
    { e0: -1.0, e1: 2.0, x: 0.5 },
  ];

  for (const c of cases) {
    gl.uniform1f(uE0Loc, c.e0);
    gl.uniform1f(uE1Loc, c.e1);
    gl.uniform1f(uXLoc, c.x);
    gl.drawArrays(gl.POINTS, 0, 1);
    const pixels = new Float32Array(4);
    gl.readPixels(0, 0, 1, 1, gl.RGBA, gl.FLOAT, pixels);
    const result = pixels[0];
    const expected = computeSmoothStep(c.e0, c.e1, c.x);
    assert(Math.abs(result - expected) < 1e-6, `smoothstep(${JSON.stringify(c)}) failed (got ${result}, expected ${expected})`);
  }
  gl.bindFramebuffer(gl.FRAMEBUFFER, null);
  gl.deleteFramebuffer(fb);
  gl.deleteTexture(tex);
});


test('Math Builtin: smoothstep vector', async (t) => {
  const gl = await webGL2({ size: { width: 1, height: 1 } });

  const vs = `#version 300 es
        in vec4 position;
        void main() { gl_Position = position; }
    `;

  const fs = `#version 300 es
        precision highp float;
        uniform vec2 u_e0;
        uniform vec2 u_e1;
        uniform vec2 u_x;
        out vec4 outColor;
        void main() {
            vec2 r = smoothstep(u_e0, u_e1, u_x);
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
  gl.uniform2f(gl.getUniformLocation(program, 'u_e0'), 0.0, -1.0);
  gl.uniform2f(gl.getUniformLocation(program, 'u_e1'), 1.0, 2.0);
  gl.uniform2f(gl.getUniformLocation(program, 'u_x'), 0.25, 0.5);

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
  const expected = computeSmoothStep(0.0, 1.0, 0.25);
  assert(Math.abs(result - expected) < 1e-6, `smoothstep(vec2).x failed (got ${result}, expected ${expected})`);
  gl.bindFramebuffer(gl.FRAMEBUFFER, null);
  gl.deleteFramebuffer(fb);
  gl.deleteTexture(tex);
});