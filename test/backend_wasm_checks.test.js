import test from 'node:test';
import assert from 'node:assert';
import { webGL2 } from '../index.js';

// These tests inspect generated WASM for integer load/compare opcodes.

test('Backend WASM checks (opcode presence)', async (t) => {
  const gl = await webGL2({ debug: 'shaders' });
  try {
    gl.viewport(0, 0, 64, 64);

    await t.test('I32Load present for integer FS inputs', () => {
      const vs = `#version 300 es
      layout(location=0) in ivec4 a; void main(){ gl_Position = vec4(0.0); }`;
      const fs = `#version 300 es
      precision highp float; flat in ivec4 a; out vec4 fragColor;
      void main(){ if (a.x == -1) fragColor = vec4(0,1,0,1); else fragColor = vec4(1,0,0,1);} `;

      const s_vs = gl.createShader(gl.VERTEX_SHADER); gl.shaderSource(s_vs, vs); gl.compileShader(s_vs); assert.ok(gl.getShaderParameter(s_vs, gl.COMPILE_STATUS));
      const s_fs = gl.createShader(gl.FRAGMENT_SHADER); gl.shaderSource(s_fs, fs); gl.compileShader(s_fs); assert.ok(gl.getShaderParameter(s_fs, gl.COMPILE_STATUS));

      const prog = gl.createProgram(); gl.attachShader(prog, s_vs); gl.attachShader(prog, s_fs); gl.linkProgram(prog);
      assert.ok(gl.getProgramParameter(prog, gl.LINK_STATUS));

      const wasm = gl.getProgramWasm(prog, gl.FRAGMENT_SHADER);
      assert.ok(wasm && wasm.length > 0);

      // Search for I32Load opcode 0x28 and I32Eq opcode 0x46
      const hasI32Load = wasm.includes(0x28);
      const hasI32Eq = wasm.includes(0x46);

      assert.ok(hasI32Load || hasI32Eq, 'WASM should contain integer load or compare opcodes');
    });

    await t.test('F32Load not used for integer-only compare', () => {
      const vs = `#version 300 es
      layout(location=0) in ivec4 a; void main(){ gl_Position = vec4(0.0); }`;
      const fs = `#version 300 es
      precision highp float; flat in ivec4 a; out vec4 fragColor;
      void main(){ if (a.x == -1) fragColor = vec4(0,1,0,1); else fragColor = vec4(1,0,0,1);} `;

      const s_vs = gl.createShader(gl.VERTEX_SHADER); gl.shaderSource(s_vs, vs); gl.compileShader(s_vs); assert.ok(gl.getShaderParameter(s_vs, gl.COMPILE_STATUS));
      const s_fs = gl.createShader(gl.FRAGMENT_SHADER); gl.shaderSource(s_fs, fs); gl.compileShader(s_fs); assert.ok(gl.getShaderParameter(s_fs, gl.COMPILE_STATUS));

      const prog = gl.createProgram(); gl.attachShader(prog, s_vs); gl.attachShader(prog, s_fs); gl.linkProgram(prog);
      assert.ok(gl.getProgramParameter(prog, gl.LINK_STATUS));

      const wasm = gl.getProgramWasm(prog, gl.FRAGMENT_SHADER);
      assert.ok(wasm && wasm.length > 0);

      // Ideally no F32Load appears before the integer loads for the attribute (allowing other helper code to use f32 later)
      const firstI32 = wasm.indexOf(0x28);
      const firstF32 = wasm.indexOf(0x2A);
      assert.ok(firstI32 !== -1, 'Expected at least one I32Load for integer varying');
      assert.ok(firstF32 === -1 || firstI32 < firstF32, 'First I32Load should occur before any F32Load (attribute loads should use I32Load)');
    });

  } finally {
    gl.destroy();
  }
});
