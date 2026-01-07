import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('output_layout_uniforms: verify uniform offsets in WASM', async (t) => {
  const gl = await webGL2({ debug: 'shaders' });
  try {
    await t.test('Uniform offset (location 2)', () => {
      const vsSource = `#version 300 es
      layout(location = 0) in vec4 pos;
      uniform vec4 u_unused1;
      uniform vec4 u_unused2;
      uniform vec4 u_data;
      void main() { gl_Position = pos + u_data; }`;
      const fsSource = `#version 300 es
      precision highp float; out vec4 fragColor; void main() { fragColor = vec4(1.0); }`;
      
      const vs = gl.createShader(gl.VERTEX_SHADER);
      gl.shaderSource(vs, vsSource);
      gl.compileShader(vs);
      if (!gl.getShaderParameter(vs, gl.COMPILE_STATUS)) {
        throw new Error("VS Compile failed: " + gl.getShaderInfoLog(vs));
      }

      const fs = gl.createShader(gl.FRAGMENT_SHADER);
      gl.shaderSource(fs, fsSource);
      gl.compileShader(fs);
      if (!gl.getShaderParameter(fs, gl.COMPILE_STATUS)) {
        throw new Error("FS Compile failed: " + gl.getShaderInfoLog(fs));
      }
      
      const prog = gl.createProgram();
      gl.attachShader(prog, vs);
      gl.attachShader(prog, fs);
      gl.linkProgram(prog);
      
      if (!gl.getProgramParameter(prog, gl.LINK_STATUS)) {
        throw new Error("Link failed: " + gl.getProgramInfoLog(prog));
      }
      
      const wasm = gl.getProgramWasm(prog, gl.VERTEX_SHADER);
      assert.ok(wasm, 'WASM should be available for linked vertex shader');
      // u_data should be at location 2 (assigned sequentially)
      // location 2 * 64 = 128
      // I32Const(128) -> 0x41 0x80 0x01
      let found = false;
      for (let i = 0; i < wasm.length - 2; i++) {
        if (wasm[i] === 0x41 && wasm[i+1] === 0x80 && wasm[i+2] === 0x01) {
          found = true;
          break;
        }
      }
      assert.ok(found, 'WASM should contain I32Const(128) [0x41 0x80 0x01] for uniform at location 2');
    });
  } finally {
    gl.destroy();
  }
});
