import test from 'node:test';
import assert from 'node:assert';
import { webGL2 } from '../index.js';

test('Triangle Interpolation Debugging', async (t) => {

    // Helper to compile program with an explicit context
    function createProgram(gl, vsSrc, fsSrc) {
      const vs = gl.createShader(gl.VERTEX_SHADER);
      gl.shaderSource(vs, vsSrc);
      gl.compileShader(vs);
      if (!gl.getShaderParameter(vs, gl.COMPILE_STATUS)) throw new Error(gl.getShaderInfoLog(vs));

      const fs = gl.createShader(gl.FRAGMENT_SHADER);
      gl.shaderSource(fs, fsSrc);
      gl.compileShader(fs);
      if (!gl.getShaderParameter(fs, gl.COMPILE_STATUS)) throw new Error(gl.getShaderInfoLog(fs));

      const p = gl.createProgram();
      gl.attachShader(p, vs);
      gl.attachShader(p, fs);
      gl.linkProgram(p);
      if (!gl.getProgramParameter(p, gl.LINK_STATUS)) throw new Error(gl.getProgramInfoLog(p));
      return p;
    }

    // Shared geometry
    const positions = new Float32Array([
      -1.0, -1.0, // V0
      3.0, -1.0, // V1
      -1.0, 3.0  // V2
    ]);

    // Test 1: Flat Int Triangle
await t.test('Flat Int Triangle', async () => {
    const gl = await webGL2();
    gl.viewport(0, 0, 10, 10);
    try {
      const vs = `#version 300 es
        layout(location=0) in vec2 a_pos;
        layout(location=1) in int a_val;
        flat out int v_val;
        void main() {
          v_val = a_val;
          gl_Position = vec4(a_pos, 0.0, 1.0);
        }`;
      const fs = `#version 300 es
        precision highp float;
        flat in int v_val;
        out vec4 fragColor;
        void main() {
          if (v_val == 30) fragColor = vec4(0.0, 1.0, 0.0, 1.0); // Green
          else fragColor = vec4(1.0, 0.0, 0.0, 1.0); // Red
        }`;
      const p = createProgram(gl, vs, fs);
      gl.useProgram(p);

      const values = new Int32Array([10, 20, 30]);

      const vao = gl.createVertexArray();
      gl.bindVertexArray(vao);

      const bufPos = gl.createBuffer();
      gl.bindBuffer(gl.ARRAY_BUFFER, bufPos);
      gl.bufferData(gl.ARRAY_BUFFER, positions, gl.STATIC_DRAW);
      gl.enableVertexAttribArray(0);
      gl.vertexAttribPointer(0, 2, gl.FLOAT, false, 0, 0);

      const bufVal = gl.createBuffer();
      gl.bindBuffer(gl.ARRAY_BUFFER, bufVal);
      gl.bufferData(gl.ARRAY_BUFFER, values, gl.STATIC_DRAW);
      gl.enableVertexAttribArray(1);
      gl.vertexAttribIPointer(1, 1, gl.INT, 0, 0);

      gl.clearColor(0, 0, 0, 0);
      gl.clear(gl.COLOR_BUFFER_BIT);
      gl.drawArrays(gl.TRIANGLES, 0, 3);

      const pixels = new Uint8Array(4);
      gl.readPixels(5, 5, 1, 1, gl.RGBA, gl.UNSIGNED_BYTE, pixels);
      assert.deepStrictEqual(Array.from(pixels), [0, 255, 0, 255]);
    } finally {
      gl.destroy();
    }
    });

    // Test 2: Flat IVec2 Triangle
await t.test('Flat IVec2 Triangle', async () => {
    const gl = await webGL2();
    gl.viewport(0, 0, 10, 10);
    try {
      const vs = `#version 300 es
        layout(location=0) in vec2 a_pos;
        layout(location=1) in ivec2 a_val;
        flat out ivec2 v_val;
        void main() {
          v_val = a_val;
          gl_Position = vec4(a_pos, 0.0, 1.0);
        }`;
      const fs = `#version 300 es
        precision highp float;
        flat in ivec2 v_val;
        out vec4 fragColor;
        void main() {
          if (v_val == ivec2(30, 31)) fragColor = vec4(0.0, 1.0, 0.0, 1.0);
          else fragColor = vec4(1.0, 0.0, 0.0, 1.0);
        }`;
      const p = createProgram(gl, vs, fs);
      gl.useProgram(p);

      const values = new Int32Array([
        10, 11,
        20, 21,
        30, 31
      ]);

      const vao = gl.createVertexArray();
      gl.bindVertexArray(vao);

      const bufPos = gl.createBuffer();
      gl.bindBuffer(gl.ARRAY_BUFFER, bufPos);
      gl.bufferData(gl.ARRAY_BUFFER, positions, gl.STATIC_DRAW);
      gl.enableVertexAttribArray(0);
      gl.vertexAttribPointer(0, 2, gl.FLOAT, false, 0, 0);

      const bufVal = gl.createBuffer();
      gl.bindBuffer(gl.ARRAY_BUFFER, bufVal);
      gl.bufferData(gl.ARRAY_BUFFER, values, gl.STATIC_DRAW);
      gl.enableVertexAttribArray(1);
      gl.vertexAttribIPointer(1, 2, gl.INT, 0, 0);

      gl.clearColor(0, 0, 0, 0);
      gl.clear(gl.COLOR_BUFFER_BIT);
      gl.drawArrays(gl.TRIANGLES, 0, 3);

      const pixels = new Uint8Array(4);
      gl.readPixels(5, 5, 1, 1, gl.RGBA, gl.UNSIGNED_BYTE, pixels);
      assert.deepStrictEqual(Array.from(pixels), [0, 255, 0, 255]);
    } finally { gl.destroy(); }
    });

    // Test 3: Flat IVec3 Triangle
await t.test('Flat IVec3 Triangle', async () => {
    const gl = await webGL2();
    gl.viewport(0, 0, 10, 10);
    try {
      const vs = `#version 300 es
        layout(location=0) in vec2 a_pos;
        layout(location=1) in ivec3 a_val;
        flat out ivec3 v_val;
        void main() {
          v_val = a_val;
          gl_Position = vec4(a_pos, 0.0, 1.0);
        }`;
      const fs = `#version 300 es
        precision highp float;
        flat in ivec3 v_val;
        out vec4 fragColor;
        void main() {
          if (v_val == ivec3(30, 31, 32)) fragColor = vec4(0.0, 1.0, 0.0, 1.0);
          else fragColor = vec4(1.0, 0.0, 0.0, 1.0);
        }`;
      const p = createProgram(gl, vs, fs);
      gl.useProgram(p);

      const values = new Int32Array([
        10, 11, 12,
        20, 21, 22,
        30, 31, 32
      ]);

      const vao = gl.createVertexArray();
      gl.bindVertexArray(vao);

      const bufPos = gl.createBuffer();
      gl.bindBuffer(gl.ARRAY_BUFFER, bufPos);
      gl.bufferData(gl.ARRAY_BUFFER, positions, gl.STATIC_DRAW);
      gl.enableVertexAttribArray(0);
      gl.vertexAttribPointer(0, 2, gl.FLOAT, false, 0, 0);

      const bufVal = gl.createBuffer();
      gl.bindBuffer(gl.ARRAY_BUFFER, bufVal);
      gl.bufferData(gl.ARRAY_BUFFER, values, gl.STATIC_DRAW);
      gl.enableVertexAttribArray(1);
      gl.vertexAttribIPointer(1, 3, gl.INT, 0, 0);

      gl.clearColor(0, 0, 0, 0);
      gl.clear(gl.COLOR_BUFFER_BIT);
      gl.drawArrays(gl.TRIANGLES, 0, 3);

      const pixels = new Uint8Array(4);
      gl.readPixels(5, 5, 1, 1, gl.RGBA, gl.UNSIGNED_BYTE, pixels);
      assert.deepStrictEqual(Array.from(pixels), [0, 255, 0, 255]);
    } finally { gl.destroy(); }
    });

    // Test 4: Flat IVec4 Triangle (Original Failing Test)
await t.test('Flat IVec4 Triangle', async () => {
    const gl = await webGL2();
    gl.viewport(0, 0, 10, 10);
    try {
      const vs = `#version 300 es
        layout(location=0) in vec2 a_pos;
        layout(location=1) in ivec4 a_val;
        flat out ivec4 v_val;
        void main() {
          v_val = a_val;
          gl_Position = vec4(a_pos, 0.0, 1.0);
        }`;
      const fs = `#version 300 es
        precision highp float;
        flat in ivec4 v_val;
        out vec4 fragColor;
        void main() {
          if (v_val == ivec4(30, 31, 32, 33)) fragColor = vec4(0.0, 1.0, 0.0, 1.0);
          else fragColor = vec4(1.0, 0.0, 0.0, 1.0);
        }`;
      const p = createProgram(gl, vs, fs);
      gl.useProgram(p);

      const values = new Int32Array([
        10, 11, 12, 13,
        20, 21, 22, 23,
        30, 31, 32, 33
      ]);

      const vao = gl.createVertexArray();
      gl.bindVertexArray(vao);

      const bufPos = gl.createBuffer();
      gl.bindBuffer(gl.ARRAY_BUFFER, bufPos);
      gl.bufferData(gl.ARRAY_BUFFER, positions, gl.STATIC_DRAW);
      gl.enableVertexAttribArray(0);
      gl.vertexAttribPointer(0, 2, gl.FLOAT, false, 0, 0);

      const bufVal = gl.createBuffer();
      gl.bindBuffer(gl.ARRAY_BUFFER, bufVal);
      gl.bufferData(gl.ARRAY_BUFFER, values, gl.STATIC_DRAW);
      gl.enableVertexAttribArray(1);
      gl.vertexAttribIPointer(1, 4, gl.INT, 0, 0);

      gl.clearColor(0, 0, 0, 0);
      gl.clear(gl.COLOR_BUFFER_BIT);
      gl.drawArrays(gl.TRIANGLES, 0, 3);

      const pixels = new Uint8Array(4);
      gl.readPixels(5, 5, 1, 1, gl.RGBA, gl.UNSIGNED_BYTE, pixels);
      assert.deepStrictEqual(Array.from(pixels), [0, 255, 0, 255]);
    } finally { gl.destroy(); }
    });

    // Test 5: Flat IVec4 Triangle (Simplified White Output)
    await t.test('Flat IVec4 Triangle (White)', async () => {
      const gl = await webGL2();
      gl.viewport(0, 0, 10, 10);
      try {
        const vs = `#version 300 es
        layout(location=0) in vec2 a_pos;
        layout(location=1) in ivec4 a_val;
        flat out ivec4 v_val;
        void main() {
          v_val = a_val;
          gl_Position = vec4(a_pos, 0.0, 1.0);
        }`;
        const fs = `#version 300 es
        precision highp float;
        flat in ivec4 v_val;
        out vec4 fragColor;
        void main() {
          fragColor = vec4(1.0, 1.0, 1.0, 1.0);
        }`;
        const p = createProgram(gl, vs, fs);
        gl.useProgram(p);

        const values = new Int32Array([
          10, 11, 12, 13,
          20, 21, 22, 23,
          30, 31, 32, 33
        ]);

        const vao = gl.createVertexArray();
        gl.bindVertexArray(vao);

        const bufPos = gl.createBuffer();
        gl.bindBuffer(gl.ARRAY_BUFFER, bufPos);
        gl.bufferData(gl.ARRAY_BUFFER, positions, gl.STATIC_DRAW);
        gl.enableVertexAttribArray(0);
        gl.vertexAttribPointer(0, 2, gl.FLOAT, false, 0, 0);

        const bufVal = gl.createBuffer();
        gl.bindBuffer(gl.ARRAY_BUFFER, bufVal);
        gl.bufferData(gl.ARRAY_BUFFER, values, gl.STATIC_DRAW);
        gl.enableVertexAttribArray(1);
        gl.vertexAttribIPointer(1, 4, gl.INT, 0, 0);

        gl.clearColor(0, 0, 0, 0);
        gl.clear(gl.COLOR_BUFFER_BIT);
        gl.drawArrays(gl.TRIANGLES, 0, 3);

        const pixels = new Uint8Array(4);
        gl.readPixels(5, 5, 1, 1, gl.RGBA, gl.UNSIGNED_BYTE, pixels);

        assert.deepStrictEqual(Array.from(pixels), [255, 255, 255, 255]);
      } finally { gl.destroy(); }
    });
});
