import { test } from 'node:test';
import assert from 'node:assert';
import { webGL2 } from '../../../index.js';

/*
 * FAIL: This test fails because `gl.vertexAttribIPointer` is not implemented.
 * 
 * Missing Functions:
 * 1. `gl.vertexAttribIPointer`: Not implemented in JS or Rust.
 */

const glEnumToString = (gl, value) => {
  for (const key in gl) {
    if (gl[key] === value) {
      return key;
    }
  }
  return `0x${value.toString(16)}`;
};

const checkError = (gl, expectedError, msg) => {
  const error = gl.getError();
  assert.strictEqual(
    error,
    expectedError,
    `${msg}: expected ${glEnumToString(gl, expectedError)}, got ${glEnumToString(gl, error)}`
  );
};

test('WebGL vertexAttribIPointer Conformance Tests', { skip: true }, async (t) => {
  const gl = await webGL2();
  assert.ok(gl, 'context does not exist');

  await t.test('Initial checks', () => {
    // vertexAttribIPointer should succeed if no buffer is bound and offset is zero
    gl.vertexAttribIPointer(0, 3, gl.INT, 0, 0);
    checkError(gl, gl.NO_ERROR, "vertexAttribIPointer should succeed if no buffer is bound and offset is zero");

    // vertexAttribIPointer should fail if no buffer is bound and offset is non-zero
    gl.vertexAttribIPointer(0, 3, gl.INT, 0, 12);
    checkError(gl, gl.INVALID_OPERATION, "vertexAttribIPointer should fail if no buffer is bound and offset is non-zero");
  });

  await t.test('Buffer bound checks', async (t) => {
    const vertexObject = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, vertexObject);
    gl.bufferData(gl.ARRAY_BUFFER, new Int32Array(0), gl.STATIC_DRAW);

    await t.test('FLOAT support', () => {
      // vertexAttribIPointer should not support FLOAT
      gl.vertexAttribIPointer(0, 1, gl.FLOAT, 0, 0);
      checkError(gl, gl.INVALID_ENUM, "vertexAttribIPointer should not support FLOAT");
    });

    const types = [
      { type: gl.BYTE, bytesPerComponent: 1 },
      { type: gl.UNSIGNED_BYTE, bytesPerComponent: 1 },
      { type: gl.SHORT, bytesPerComponent: 2 },
      { type: gl.UNSIGNED_SHORT, bytesPerComponent: 2 },
      { type: gl.INT, bytesPerComponent: 4 },
      { type: gl.UNSIGNED_INT, bytesPerComponent: 4 },
    ];

    for (const info of types) {
      await t.test(`Type: ${glEnumToString(gl, info.type)}`, async (t) => {
        const actual = [];
        const expected = [];

        for (let size = 1; size <= 4; ++size) {
          const offsetSet = [
            0,
            1,
            info.bytesPerComponent - 1,
            info.bytesPerComponent,
            info.bytesPerComponent + 1,
            info.bytesPerComponent * 2
          ];

          for (const offset of offsetSet) {
            for (const stride of offsetSet) {
              let expectedErr = gl.NO_ERROR;

              if (offset % info.bytesPerComponent !== 0) {
                expectedErr = gl.INVALID_OPERATION;
              }
              if (stride % info.bytesPerComponent !== 0) {
                expectedErr = gl.INVALID_OPERATION;
              }

              gl.vertexAttribIPointer(0, size, info.type, stride, offset);
              actual.push({
                size, stride, offset,
                error: glEnumToString(gl, gl.getError())
              });
              expected.push({
                size, stride, offset,
                error: glEnumToString(gl, expectedErr)
              });
            }

            const maxStride = Math.floor(255 / info.bytesPerComponent) * info.bytesPerComponent;

            if (offset === 0) {
              // at stride limit
              gl.vertexAttribIPointer(0, size, info.type, maxStride, offset);
              actual.push({
                size, stride: maxStride, offset, case: 'at stride limit',
                error: glEnumToString(gl, gl.getError())
              });
              expected.push({
                size, stride: maxStride, offset, case: 'at stride limit',
                error: glEnumToString(gl, gl.NO_ERROR)
              });

              // over stride limit
              gl.vertexAttribIPointer(0, size, info.type, maxStride + info.bytesPerComponent, offset);
              actual.push({
                size, stride: maxStride + info.bytesPerComponent, offset, case: 'over stride limit',
                error: glEnumToString(gl, gl.getError())
              });
              expected.push({
                size, stride: maxStride + info.bytesPerComponent, offset, case: 'over stride limit',
                error: glEnumToString(gl, gl.INVALID_VALUE)
              });
            }
          }
        }
        assert.deepStrictEqual(actual, expected);
      });
    }
  });
});
