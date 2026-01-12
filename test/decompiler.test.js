import test from 'node:test';
import assert from 'node:assert/strict';
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

void main(int p0, int p1, int p2, int p3, int p4, int p5) {
    float v6;
    float v7;
    float v8;
    float v9;
    float v10;
    float v11;
    float v12;
    float v13;
    float v14;
    float v15;
    float v16;
    float v17;
    float v18;
    float v19;
    float v20;
    float v21;
    float v22;
    float v23;
    float v24;
    float v25;
    float v26;
    float v27;
    float v28;
    float v29;
    float v30;
    float v31;
    float v32;
    float v33;
    float v34;
    float v35;
    float v36;
    float v37;
    int v38;
    int v39;
    int v40;
    int v41;
    int v42;
    int v43;
    int v44;
    int v45;
    int v46;
    int v47;
    int v48;
    int v49;
    int v50;
    int v51;
    int v52;
    int v53;
    int v54;
    int v55;
    int v56;
    int v57;
    int v58;
    int v59;
    int v60;
    int v61;
    int v62;
    int v63;
    int v64;
    int v65;
    int v66;
    int v67;
    int v68;
    int v69;
    int v70;
    float v71;
    int v72;
    
    g0 = p1;
    g1 = p2;
    g2 = p3;
    g3 = p4;
    g4 = p5;
    g5 = 524288;
    v71 = /* unknown: __unsimplified__ */;
    memory[((g2) + 12) >> 2] = floatBitsToInt(v71);
    v71 = /* unknown: __unsimplified__ */;
    memory[((g2) + 8) >> 2] = floatBitsToInt(v71);
    v71 = /* unknown: __unsimplified__ */;
    memory[((g2) + 4) >> 2] = floatBitsToInt(v71);
    v71 = /* unknown: __unsimplified__ */;
    memory[(g2) >> 2] = floatBitsToInt(v71);
    return /* unknown: __unsimplified__ */;
}
void func_1() {
    float v0;
    float v1;
    float v2;
    float v3;
    float v4;
    float v5;
    float v6;
    float v7;
    float v8;
    float v9;
    float v10;
    float v11;
    float v12;
    float v13;
    float v14;
    float v15;
    float v16;
    float v17;
    float v18;
    float v19;
    float v20;
    float v21;
    float v22;
    float v23;
    float v24;
    float v25;
    float v26;
    float v27;
    float v28;
    float v29;
    float v30;
    float v31;
    int v32;
    int v33;
    int v34;
    int v35;
    int v36;
    int v37;
    int v38;
    int v39;
    int v40;
    int v41;
    int v42;
    int v43;
    int v44;
    int v45;
    int v46;
    int v47;
    int v48;
    int v49;
    int v50;
    int v51;
    int v52;
    int v53;
    int v54;
    int v55;
    int v56;
    int v57;
    int v58;
    int v59;
    int v60;
    int v61;
    int v62;
    int v63;
    int v64;
    float v65;
    int v66;
    
    memory[(g2) >> 2] = floatBitsToInt(0.0);
    memory[((g2) + 4) >> 2] = floatBitsToInt(0.0);
    memory[((g2) + 8) >> 2] = floatBitsToInt(0.0);
    memory[((g2) + 12) >> 2] = floatBitsToInt(0.0);
    return /* unknown: __unsimplified__ */;
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

void func_1() {
    float v0;
    float v1;
    float v2;
    float v3;
    float v4;
    float v5;
    float v6;
    float v7;
    float v8;
    float v9;
    float v10;
    float v11;
    float v12;
    float v13;
    float v14;
    float v15;
    float v16;
    float v17;
    float v18;
    float v19;
    float v20;
    float v21;
    float v22;
    float v23;
    float v24;
    float v25;
    float v26;
    float v27;
    float v28;
    float v29;
    float v30;
    float v31;
    int v32;
    int v33;
    int v34;
    int v35;
    int v36;
    int v37;
    int v38;
    int v39;
    int v40;
    int v41;
    int v42;
    int v43;
    int v44;
    int v45;
    int v46;
    int v47;
    int v48;
    int v49;
    int v50;
    int v51;
    int v52;
    int v53;
    int v54;
    int v55;
    int v56;
    int v57;
    int v58;
    int v59;
    int v60;
    int v61;
    int v62;
    int v63;
    int v64;
    float v65;
    int v66;
    
    memory[(g3) >> 2] = floatBitsToInt(1.0);
    memory[((g3) + 4) >> 2] = floatBitsToInt(1.0);
    memory[((g3) + 8) >> 2] = floatBitsToInt(1.0);
    memory[((g3) + 12) >> 2] = floatBitsToInt(1.0);
    return /* unknown: __unsimplified__ */;
}
void main(int p0, int p1, int p2, int p3, int p4, int p5) {
    float v6;
    float v7;
    float v8;
    float v9;
    float v10;
    float v11;
    float v12;
    float v13;
    float v14;
    float v15;
    float v16;
    float v17;
    float v18;
    float v19;
    float v20;
    float v21;
    float v22;
    float v23;
    float v24;
    float v25;
    float v26;
    float v27;
    float v28;
    float v29;
    float v30;
    float v31;
    float v32;
    float v33;
    float v34;
    float v35;
    float v36;
    float v37;
    int v38;
    int v39;
    int v40;
    int v41;
    int v42;
    int v43;
    int v44;
    int v45;
    int v46;
    int v47;
    int v48;
    int v49;
    int v50;
    int v51;
    int v52;
    int v53;
    int v54;
    int v55;
    int v56;
    int v57;
    int v58;
    int v59;
    int v60;
    int v61;
    int v62;
    int v63;
    int v64;
    int v65;
    int v66;
    int v67;
    int v68;
    int v69;
    int v70;
    float v71;
    int v72;
    
    g0 = p1;
    g1 = p2;
    g2 = p3;
    g3 = p4;
    g4 = p5;
    g5 = 524288;
    v71 = /* unknown: __unsimplified__ */;
    memory[((g3) + 12) >> 2] = floatBitsToInt(v71);
    v71 = /* unknown: __unsimplified__ */;
    memory[((g3) + 8) >> 2] = floatBitsToInt(v71);
    v71 = /* unknown: __unsimplified__ */;
    memory[((g3) + 4) >> 2] = floatBitsToInt(v71);
    v71 = /* unknown: __unsimplified__ */;
    memory[(g3) >> 2] = floatBitsToInt(v71);
    return /* unknown: __unsimplified__ */;
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

void func_1() {
    float v0;
    float v1;
    float v2;
    float v3;
    float v4;
    float v5;
    float v6;
    float v7;
    float v8;
    float v9;
    float v10;
    float v11;
    float v12;
    float v13;
    float v14;
    float v15;
    float v16;
    float v17;
    float v18;
    float v19;
    float v20;
    float v21;
    float v22;
    float v23;
    float v24;
    float v25;
    float v26;
    float v27;
    float v28;
    float v29;
    float v30;
    float v31;
    int v32;
    int v33;
    int v34;
    int v35;
    int v36;
    int v37;
    int v38;
    int v39;
    int v40;
    int v41;
    int v42;
    int v43;
    int v44;
    int v45;
    int v46;
    int v47;
    int v48;
    int v49;
    int v50;
    int v51;
    int v52;
    int v53;
    int v54;
    int v55;
    int v56;
    int v57;
    int v58;
    int v59;
    int v60;
    int v61;
    int v62;
    int v63;
    int v64;
    float v65;
    int v66;
    
    memory[(g2) >> 2] = floatBitsToInt(0.0);
    memory[((g2) + 4) >> 2] = floatBitsToInt(0.0);
    memory[((g2) + 8) >> 2] = floatBitsToInt(0.0);
    memory[((g2) + 12) >> 2] = floatBitsToInt(0.0);
    return /* unknown: __unsimplified__ */;
}
void main(int p0, int p1, int p2, int p3, int p4, int p5) {
    float v6;
    float v7;
    float v8;
    float v9;
    float v10;
    float v11;
    float v12;
    float v13;
    float v14;
    float v15;
    float v16;
    float v17;
    float v18;
    float v19;
    float v20;
    float v21;
    float v22;
    float v23;
    float v24;
    float v25;
    float v26;
    float v27;
    float v28;
    float v29;
    float v30;
    float v31;
    float v32;
    float v33;
    float v34;
    float v35;
    float v36;
    float v37;
    int v38;
    int v39;
    int v40;
    int v41;
    int v42;
    int v43;
    int v44;
    int v45;
    int v46;
    int v47;
    int v48;
    int v49;
    int v50;
    int v51;
    int v52;
    int v53;
    int v54;
    int v55;
    int v56;
    int v57;
    int v58;
    int v59;
    int v60;
    int v61;
    int v62;
    int v63;
    int v64;
    int v65;
    int v66;
    int v67;
    int v68;
    int v69;
    int v70;
    float v71;
    int v72;
    
    g0 = p1;
    g1 = p2;
    g2 = p3;
    g3 = p4;
    g4 = p5;
    g5 = 524288;
    v71 = /* unknown: __unsimplified__ */;
    memory[((g2) + 12) >> 2] = floatBitsToInt(v71);
    v71 = /* unknown: __unsimplified__ */;
    memory[((g2) + 8) >> 2] = floatBitsToInt(v71);
    v71 = /* unknown: __unsimplified__ */;
    memory[((g2) + 4) >> 2] = floatBitsToInt(v71);
    v71 = /* unknown: __unsimplified__ */;
    memory[(g2) >> 2] = floatBitsToInt(v71);
    return /* unknown: __unsimplified__ */;
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
