import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('output_layout_offsets: verify vertex attribute and fragment varying offsets in WASM', async (t) => {
  const gl = await webGL2({ debug: 'shaders' });
  try {
    await t.test('Vertex attribute offset (location 1)', () => {
      const vsSource = `#version 300 es
      layout(location = 1) in vec4 a_pos;
      void main() { gl_Position = a_pos; }`;
      const fsSource = `#version 300 es
      precision highp float; out vec4 fragColor; void main() { fragColor = vec4(1.0); }`;
      
      const vs = gl.createShader(gl.VERTEX_SHADER);
      gl.shaderSource(vs, vsSource);
      gl.compileShader(vs);
      
      const fs = gl.createShader(gl.FRAGMENT_SHADER);
      gl.shaderSource(fs, fsSource);
      gl.compileShader(fs);
      
      const prog = gl.createProgram();
      gl.attachShader(prog, vs);
      gl.attachShader(prog, fs);
      gl.linkProgram(prog);
      
      if (!gl.getProgramParameter(prog, gl.LINK_STATUS)) {
        throw new Error("Link failed: " + gl.getProgramInfoLog(prog));
      }
      
      const wasm = gl.getProgramWasm(prog, gl.VERTEX_SHADER);
      assert.ok(wasm, 'WASM should be available for linked vertex shader');
      // location 1 * 64 = 64 
      // Signed LEB128 for 64 is 0xC0 0x00 (because bit 7 is 1, so it needs another byte to keep sign 0)
      // We expect I32Const(64) -> 0x41 0xC0 0x00
      let found = false;
      for (let i = 0; i < wasm.length - 2; i++) {
        if (wasm[i] === 0x41 && wasm[i+1] === 0xC0 && wasm[i+2] === 0x00) {
          found = true;
          break;
        }
      }
      assert.ok(found, 'WASM should contain I32Const(64) [0x41 0xC0 0x00] for vertex attribute at location 1');
    });

    await t.test('Fragment varying offset (location 1)', () => {
      const vs = `#version 300 es
      void main() { gl_Position = vec4(0.0); }`;
      const fs = `#version 300 es
      precision highp float;
      layout(location = 1) in vec4 v_color;
      out vec4 fragColor;
      void main() { fragColor = v_color; }`;
      
      const s_vs = gl.createShader(gl.VERTEX_SHADER); gl.shaderSource(s_vs, vs); gl.compileShader(s_vs);
      const s_fs = gl.createShader(gl.FRAGMENT_SHADER); gl.shaderSource(s_fs, fs); gl.compileShader(s_fs);
      
      const prog = gl.createProgram();
      gl.attachShader(prog, s_vs);
      gl.attachShader(prog, s_fs);
      gl.linkProgram(prog);
      
      if (!gl.getProgramParameter(prog, gl.LINK_STATUS)) {
        throw new Error("Link failed: " + gl.getProgramInfoLog(prog));
      }
      
      const wasm = gl.getProgramWasm(prog, gl.FRAGMENT_SHADER);
      assert.ok(wasm, 'WASM should be available for linked fragment shader');
      // location 1 varying offset = (1 + 2) * 16 = 48 (0x30)
      // We expect I32Const(48) -> 0x41 0x30
      let found = false;
      for (let i = 0; i < wasm.length - 1; i++) {
        if (wasm[i] === 0x41 && wasm[i+1] === 0x30) {
          found = true;
          break;
        }
      }
      assert.ok(found, 'WASM should contain I32Const(48) for fragment varying at location 1');
    });
  } finally {
    gl.destroy();
  }
});
