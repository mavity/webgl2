
import { test } from 'node:test';
import assert from 'node:assert';
import { webGL2 } from '../../../index.js';

/*
 * FAIL: This test fails due to missing API implementations in `src/webgl2_context.js`.
 * 
 * Missing Functions:
 * 1. `gl.vertexAttrib4fv(index, values)`: Not implemented.
 * 2. `gl.vertexAttribIPointer(index, size, type, stride, offset)`: Not implemented.
 * 
 * These functions are required to set up the vertex attributes for the test.
 */

const vShaderSource = `#version 300 es
layout(location=0) in ivec4 aPosition;
layout(location=1) in vec4 aColor;
out vec4 vColor;
void main()
{
    gl_Position = vec4(aPosition);
    vColor = aColor;
}
`;

const vShaderUnsignedSource = `#version 300 es
layout(location=0) in uvec4 aPosition;
layout(location=1) in vec4 aColor;
out vec4 vColor;
void main()
{
    gl_Position = vec4(aPosition);
    vColor = aColor;
}
`;

const fShaderSource = `#version 300 es
precision mediump float;
in vec4 vColor;
layout(location=0) out vec4 oColor;
void main()
{
    oColor = vColor;
}
`;

function createProgram(gl, vsSource, fsSource) {
  const vs = gl.createShader(gl.VERTEX_SHADER);
  gl.shaderSource(vs, vsSource);
  gl.compileShader(vs);
  if (!gl.getShaderParameter(vs, gl.COMPILE_STATUS)) {
    throw new Error(`Vertex shader compilation failed: ${gl.getShaderInfoLog(vs)}`);
  }

  const fs = gl.createShader(gl.FRAGMENT_SHADER);
  gl.shaderSource(fs, fsSource);
  gl.compileShader(fs);
  if (!gl.getShaderParameter(fs, gl.COMPILE_STATUS)) {
    throw new Error(`Fragment shader compilation failed: ${gl.getShaderInfoLog(fs)}`);
  }

  const prog = gl.createProgram();
  gl.attachShader(prog, vs);
  gl.attachShader(prog, fs);
  gl.linkProgram(prog);
  if (!gl.getProgramParameter(prog, gl.LINK_STATUS)) {
    throw new Error(`Program linking failed: ${gl.getProgramInfoLog(prog)}`);
  }
  return prog;
}

function checkCanvasRect(gl, x, y, width, height, expectedColor, msg) {
  const buf = new Uint8Array(width * height * 4);
  gl.readPixels(x, y, width, height, gl.RGBA, gl.UNSIGNED_BYTE, buf);

  for (let i = 0; i < width * height; i++) {
    const offset = i * 4;
    const pixel = [buf[offset], buf[offset + 1], buf[offset + 2], buf[offset + 3]];
    assert.deepStrictEqual(pixel, expectedColor,
      `${msg} at (${x}, ${y}) pixel ${i}: expected ${expectedColor}, got ${pixel}`);
  }
}

function glEnumToString(gl, value) {
  for (const key in gl) {
    if (gl[key] === value) {
      return key;
    }
  }
  return `0x${value.toString(16)}`;
}

test('vertexAttribIPointer offsets tests', async (t) => {
  const gl = await webGL2({ size: { width: 50, height: 50 } });

  const program = createProgram(gl, vShaderSource, fShaderSource);
  const program_unsigned = createProgram(gl, vShaderUnsignedSource, fShaderSource);

  const tests = [
    {
      data: new Int32Array([0, 1, 0, 1, 0, 0, 0, 0, 0]),
      type: gl.INT,
      componentSize: 4,
    },
    {
      data: new Uint32Array([0, 1, 0, 1, 0, 0, 0, 0, 0]),
      type: gl.UNSIGNED_INT,
      componentSize: 4,
    },
    {
      data: new Uint16Array([0, 32767, 0, 32767, 0, 0, 0, 0, 0]),
      type: gl.SHORT,
      componentSize: 2,
    },
    {
      data: new Uint16Array([0, 65535, 0, 65535, 0, 0, 0, 0, 0]),
      type: gl.UNSIGNED_SHORT,
      componentSize: 2,
    },
    {
      data: new Uint8Array([0, 127, 0, 127, 0, 0, 0, 0, 0]),
      type: gl.BYTE,
      componentSize: 1,
    },
    {
      data: new Uint8Array([0, 1, 0, 1, 0, 0, 0, 0, 0]),
      type: gl.UNSIGNED_BYTE,
      componentSize: 1,
    }
  ];

  const vertexObject = gl.createBuffer();
  gl.bindBuffer(gl.ARRAY_BUFFER, vertexObject);
  gl.bufferData(gl.ARRAY_BUFFER, 1024, gl.STATIC_DRAW);
  gl.enableVertexAttribArray(0);

  const kNumVerts = 3;
  const kNumComponents = 3;
  let count = 0;

  for (const testCase of tests) {
    await t.test(glEnumToString(gl, testCase.type), async (t) => {
      for (let oo = 0; oo < 3; ++oo) {
        for (let ss = 0; ss < 3; ++ss) {
          const offset = (oo + 1) * testCase.componentSize;
          const stride = testCase.componentSize * kNumComponents + testCase.componentSize * ss;

          await t.test(`offset: ${offset}, stride: ${stride}`, () => {
            const color = (count % 2) ? [1, 0, 0, 1] : [0, 1, 0, 1];

            // Note: In the original test, they switch programs based on type.
            if (testCase.type === gl.INT || testCase.type === gl.SHORT || testCase.type === gl.BYTE) {
              gl.useProgram(program);
            } else {
              gl.useProgram(program_unsigned);
            }

            gl.vertexAttrib4fv(1, color);

            // Construct data with stride/padding
            const dataSize = testCase.componentSize * kNumVerts * kNumComponents + stride * (kNumVerts - 1);
            const data = new Uint8Array(dataSize);
            const view = new Uint8Array(testCase.data.buffer);
            const size = testCase.componentSize * kNumComponents;

            for (let jj = 0; jj < kNumVerts; ++jj) {
              const off1 = jj * size;
              const off2 = jj * stride;
              for (let zz = 0; zz < size; ++zz) {
                data[off2 + zz] = view[off1 + zz];
              }
            }

            gl.bufferSubData(gl.ARRAY_BUFFER, offset, data);

            // The critical call
            gl.vertexAttribIPointer(0, 3, testCase.type, stride, offset);

            gl.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT);
            gl.drawArrays(gl.TRIANGLES, 0, 3);

            const black = [0, 0, 0, 0]; // Alpha 0 because clear color is default 0,0,0,0
            const other = [color[0] * 255, color[1] * 255, color[2] * 255, color[3] * 255];
            const otherMsg = "should be " + ((count % 2) ? "red" : "green");

            checkCanvasRect(gl, 0, 0, 1, 1, black, "should be black");
            checkCanvasRect(gl, 0, 49, 1, 1, black, "should be black");
            checkCanvasRect(gl, 26, 40, 1, 1, other, otherMsg);
            checkCanvasRect(gl, 26, 27, 1, 1, other, otherMsg);
            checkCanvasRect(gl, 40, 27, 1, 1, other, otherMsg);
          });
          count++;
        }
      }
    });
  }
});
