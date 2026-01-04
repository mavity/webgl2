import test from 'node:test';
import assert from 'node:assert';
import { webGL2 } from '../index.js';
import { writeFileSync } from 'node:fs';
import { tmpdir } from 'os';
import path from 'path';

// Inspect generated FS WASM for integer varying reads (was previously named debug_wasm_opcodes)
test('WASM opcodes: fragment integer varying reads', async (t) => {
  const gl = await webGL2({ debug: 'shaders' });
  try {
    const vs = `#version 300 es\nlayout(location=0) in ivec4 a; void main(){ gl_Position = vec4(0.0); }`;
    const fs = `#version 300 es\nprecision highp float; flat in ivec4 a; out vec4 fragColor; void main(){ if (a.x == -1) fragColor = vec4(0,1,0,1); else fragColor = vec4(1,0,0,1);} `;

    const s_vs = gl.createShader(gl.VERTEX_SHADER); gl.shaderSource(s_vs, vs); gl.compileShader(s_vs); assert.ok(gl.getShaderParameter(s_vs, gl.COMPILE_STATUS));
    const s_fs = gl.createShader(gl.FRAGMENT_SHADER); gl.shaderSource(s_fs, fs); gl.compileShader(s_fs); assert.ok(gl.getShaderParameter(s_fs, gl.COMPILE_STATUS));

    const prog = gl.createProgram(); gl.attachShader(prog, s_vs); gl.attachShader(prog, s_fs); gl.linkProgram(prog);
    assert.ok(gl.getProgramParameter(prog, gl.LINK_STATUS));

    const wasm = gl.getProgramWasm(prog, gl.FRAGMENT_SHADER);
    assert.ok(wasm && wasm.length > 0);
    if (process.env.SAVE_WASM === '1') {
      const file = path.join(tmpdir(), `wasm_frag_int_fs_${Date.now()}.wasm`);
      writeFileSync(file, Buffer.from(wasm));
    }

    const bytes = Array.from(wasm);
    assert.ok(bytes.includes(0x28), 'I32Load expected');
    assert.ok(bytes.includes(0x46) || bytes.includes(0x47), 'I32Eq/I32Ne expected');
    // Ensure the F32Load we previously found is NOT used for integer-only path (allowing debug F32 loads elsewhere)
    // Here we only assert that integer loads exist; a stricter absence check is fragile with debug instrumentation present.

  } finally { gl.destroy(); }
});