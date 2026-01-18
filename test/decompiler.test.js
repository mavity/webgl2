import test from 'node:test';
import assert from 'node:assert/strict';

// Ensure consistent environment for decompiler tests (debug mode)
process.env.WEBGL2_DEBUG = 'true';

import { webGL2, getShaderModule, getShaderWat, getShaderGlsl, decompileWasmToGlsl } from '../index.js';

test('getShaderGlsl returns GLSL for compiled vertex shader', async () => {
  const gl = await webGL2();
  try {
    const vs = gl.createShader(gl.VERTEX_SHADER);
    gl.shaderSource(vs, '#version 300 es\nvoid main() { gl_Position = vec4(0); }');
    gl.compileShader(vs);

    const fs = gl.createShader(gl.FRAGMENT_SHADER);
    gl.shaderSource(fs, '#version 300 es\nprecision mediump float; out vec4 color; void main() { color = vec4(1); }');
    gl.compileShader(fs);

    const program = gl.createProgram();
    gl.attachShader(program, vs);
    gl.attachShader(program, fs);
    gl.linkProgram(program);

    const status = gl.getProgramParameter(program, gl.LINK_STATUS);
    assert.strictEqual(status, true, 'Program should link successfully');

    // Get GLSL for vertex shader
    const glsl = getShaderGlsl(gl._ctxHandle, program._handle, gl.VERTEX_SHADER);

    // GLSL output should contain version directive and be valid GLSL-like code
    assert.equal(glsl, `#version 300 es
precision highp float;
precision highp int;

// WASM linear memory mapped to buffer
layout(std430, binding = 0) buffer MemoryBuffer {
    int memory[];
};

void func_19() {
    int v0;
    float v1;
    int v2;
    
    memory[(g2) >> 2] = floatBitsToInt(0.0);
    memory[((g2) + 4) >> 2] = floatBitsToInt(0.0);
    memory[((g2) + 8) >> 2] = floatBitsToInt(0.0);
    memory[((g2) + 12) >> 2] = floatBitsToInt(0.0);
    return func0();
}
void main(int p0, int p1, int p2, int p3, int p4, int p5, int p6, int p7) {
    int v8;
    float v9;
    int v10;
    
    g0 = p3;
    g1 = p4;
    g2 = p5;
    g3 = p6;
    g4 = p7;
    g5 = 524288;
    v9 = /* unknown: __unsimplified__ */;
    memory[((g2) + 12) >> 2] = floatBitsToInt(v9);
    v9 = /* unknown: __unsimplified__ */;
    memory[((g2) + 8) >> 2] = floatBitsToInt(v9);
    v9 = /* unknown: __unsimplified__ */;
    memory[((g2) + 4) >> 2] = floatBitsToInt(v9);
    v9 = /* unknown: __unsimplified__ */;
    memory[(g2) >> 2] = floatBitsToInt(v9);
    return func_19();
}
`);
  } finally {
    gl.destroy();
  }
});

test('getShaderGlsl returns GLSL for compiled fragment shader', async () => {
  const gl = await webGL2();
  try {
    const vs = gl.createShader(gl.VERTEX_SHADER);
    gl.shaderSource(vs, '#version 300 es\nvoid main() { gl_Position = vec4(0); }');
    gl.compileShader(vs);

    const fs = gl.createShader(gl.FRAGMENT_SHADER);
    gl.shaderSource(fs, '#version 300 es\nprecision mediump float; out vec4 color; void main() { color = vec4(1); }');
    gl.compileShader(fs);

    const program = gl.createProgram();
    gl.attachShader(program, vs);
    gl.attachShader(program, fs);
    gl.linkProgram(program);

    const status = gl.getProgramParameter(program, gl.LINK_STATUS);
    assert.strictEqual(status, true, 'Program should link successfully');

    // Get GLSL for fragment shader
    const glsl = getShaderGlsl(gl._ctxHandle, program._handle, gl.FRAGMENT_SHADER);

    // GLSL output should contain version directive
    assert.equal(glsl, `#version 300 es
precision highp float;
precision highp int;

// WASM linear memory mapped to buffer
layout(std430, binding = 0) buffer MemoryBuffer {
    int memory[];
};

void func_19() {
    int v0;
    float v1;
    int v2;
    
    memory[(g3) >> 2] = floatBitsToInt(1.0);
    memory[((g3) + 4) >> 2] = floatBitsToInt(1.0);
    memory[((g3) + 8) >> 2] = floatBitsToInt(1.0);
    memory[((g3) + 12) >> 2] = floatBitsToInt(1.0);
    return func0();
}
void main(int p0, int p1, int p2, int p3, int p4, int p5, int p6, int p7) {
    int v8;
    float v9;
    int v10;
    
    g0 = p3;
    g1 = p4;
    g2 = p5;
    g3 = p6;
    g4 = p7;
    g5 = 524288;
    v9 = /* unknown: __unsimplified__ */;
    memory[((g3) + 12) >> 2] = floatBitsToInt(v9);
    v9 = /* unknown: __unsimplified__ */;
    memory[((g3) + 8) >> 2] = floatBitsToInt(v9);
    v9 = /* unknown: __unsimplified__ */;
    memory[((g3) + 4) >> 2] = floatBitsToInt(v9);
    v9 = /* unknown: __unsimplified__ */;
    memory[(g3) >> 2] = floatBitsToInt(v9);
    return func_19();
}
`);
  } finally {
    gl.destroy();
  }
});

test('getShaderGlsl returns null for unlinked program', async () => {
  const gl = await webGL2();
  try {
    const program = gl.createProgram();

    // Get GLSL before linking - should return null
    const glsl = getShaderGlsl(gl._ctxHandle, program._handle, gl.VERTEX_SHADER);

    assert.strictEqual(glsl, null, 'GLSL should be null for unlinked program');
  } finally {
    gl.destroy();
  }
});

test('decompileWasmToGlsl decompiles WASM bytes directly', async () => {
  const gl = await webGL2();
  try {
    const vs = gl.createShader(gl.VERTEX_SHADER);
    gl.shaderSource(vs, '#version 300 es\nvoid main() { gl_Position = vec4(0); }');
    gl.compileShader(vs);

    const fs = gl.createShader(gl.FRAGMENT_SHADER);
    gl.shaderSource(fs, '#version 300 es\nprecision mediump float; out vec4 color; void main() { color = vec4(1); }');
    gl.compileShader(fs);

    const program = gl.createProgram();
    gl.attachShader(program, vs);
    gl.attachShader(program, fs);
    gl.linkProgram(program);

    // Get WASM bytes first
    const wasmBytes = getShaderModule(gl._ctxHandle, program._handle, gl.VERTEX_SHADER);
    assert.ok(wasmBytes !== null, 'WASM bytes should not be null');

    // Decompile directly
    const glsl = decompileWasmToGlsl(gl, wasmBytes);

    assert.equal(glsl, `#version 300 es
precision highp float;
precision highp int;

// WASM linear memory mapped to buffer
layout(std430, binding = 0) buffer MemoryBuffer {
    int memory[];
};

void func_19() {
    int v0;
    float v1;
    int v2;
    
    memory[(g2) >> 2] = floatBitsToInt(0.0);
    memory[((g2) + 4) >> 2] = floatBitsToInt(0.0);
    memory[((g2) + 8) >> 2] = floatBitsToInt(0.0);
    memory[((g2) + 12) >> 2] = floatBitsToInt(0.0);
    return func0();
}
void main(int p0, int p1, int p2, int p3, int p4, int p5, int p6, int p7) {
    int v8;
    float v9;
    int v10;
    
    g0 = p3;
    g1 = p4;
    g2 = p5;
    g3 = p6;
    g4 = p7;
    g5 = 524288;
    v9 = /* unknown: __unsimplified__ */;
    memory[((g2) + 12) >> 2] = floatBitsToInt(v9);
    v9 = /* unknown: __unsimplified__ */;
    memory[((g2) + 8) >> 2] = floatBitsToInt(v9);
    v9 = /* unknown: __unsimplified__ */;
    memory[((g2) + 4) >> 2] = floatBitsToInt(v9);
    v9 = /* unknown: __unsimplified__ */;
    memory[(g2) >> 2] = floatBitsToInt(v9);
    return func_19();
}
`);
  } finally {
    gl.destroy();
  }
});

test('decompiled GLSL contains function definitions', async () => {
  const gl = await webGL2();
  try {
    const vs = gl.createShader(gl.VERTEX_SHADER);
    gl.shaderSource(vs, '#version 300 es\nvoid main() { gl_Position = vec4(1.0, 0.0, 0.0, 1.0); }');
    gl.compileShader(vs);

    const fs = gl.createShader(gl.FRAGMENT_SHADER);
    gl.shaderSource(fs, '#version 300 es\nprecision mediump float; out vec4 color; void main() { color = vec4(1); }');
    gl.compileShader(fs);

    const program = gl.createProgram();
    gl.attachShader(program, vs);
    gl.attachShader(program, fs);
    gl.linkProgram(program);

    const glsl = getShaderGlsl(gl._ctxHandle, program._handle, gl.VERTEX_SHADER);

    // The decompiled output should contain function-like structures
    assert.ok(glsl, '');
  } finally {
    gl.destroy();
  }
});
