import test from 'node:test';
import assert from 'node:assert';
import { webGL2 } from '../index.js';
import { writeFileSync } from 'node:fs';

// Inspect generated FS WASM for integer uniform reads (opcode presence)
test('WASM opcodes: uniform integer reads', async (t) => {
  const gl = await webGL2();
  try {
    const vs = `#version 300 es\nvoid main(){ gl_Position = vec4(0.0); gl_PointSize = 1.0; }`;
    const fs = `#version 300 es\nprecision highp float; uniform int u0; uniform int u1; uniform int u2; uniform int u3; out vec4 fragColor;\nvoid main(){ if (u0 != -1) { fragColor = vec4(1,0,0,1); return; } if (u1 != 2) { fragColor=vec4(1,0,0,1); return;} if (u2 != -3) { fragColor=vec4(1,0,0,1); return;} if (u3 != 4) { fragColor=vec4(1,0,0,1); return;} fragColor = vec4(0,1,0,1); }`;

    const s_vs = gl.createShader(gl.VERTEX_SHADER); gl.shaderSource(s_vs, vs); gl.compileShader(s_vs); assert.ok(gl.getShaderParameter(s_vs, gl.COMPILE_STATUS));
    const s_fs = gl.createShader(gl.FRAGMENT_SHADER); gl.shaderSource(s_fs, fs); gl.compileShader(s_fs); assert.ok(gl.getShaderParameter(s_fs, gl.COMPILE_STATUS));
    const prog = gl.createProgram(); gl.attachShader(prog, s_vs); gl.attachShader(prog, s_fs); gl.linkProgram(prog);
    assert.ok(gl.getProgramParameter(prog, gl.LINK_STATUS));

    const wasm = gl.getProgramWasm(prog, gl.FRAGMENT_SHADER);
    assert.ok(wasm && wasm.length > 0);
    if (process.env.SAVE_WASM === '1') {
      const { tmpdir } = await import('node:os');
      const { join } = await import('node:path');
      const file = join(tmpdir(), `wasm_uniform_fs_${Date.now()}.wasm`);
      writeFileSync(file, Buffer.from(wasm));
    }

    // Scan for I32Load/I32Ne opcodes
    const hasI32Load = Array.from(wasm).includes(0x28);
    const hasI32Ne = Array.from(wasm).includes(0x47);
    assert.ok(hasI32Load && hasI32Ne, 'Expected integer loads and integer comparison opcodes');

  } finally { gl.destroy(); }
});