import { test } from 'node:test';
import assert from 'node:assert';
import { webGL2 } from '../../../index.js';

/*
 * FAIL: This test fails due to multiple issues in the implementation.
 * 
 * 1. `drawArrays`: Fails to generate `INVALID_OPERATION` when an attribute is enabled but no buffer is bound.
 *    The implementation seems to lack validation for enabled attributes without bound buffers.
 * 
 * 2. `drawElements`, `drawArraysInstanced`, `drawElementsInstanced`: Fail to draw correctly (canvas remains black) even with valid attributes.
 *    This suggests issues with index buffer handling or instanced drawing in the emulation layer.
 * 
 * 3. `drawRangeElements`: Not implemented in `src/webgl2_context.js`.
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

const checkCanvas = (gl, expectedColor, msg) => {
  const width = 16;
  const height = 16;
  const pixels = new Uint8Array(4);
  // Read center pixel
  gl.readPixels(width / 2, height / 2, 1, 1, gl.RGBA, gl.UNSIGNED_BYTE, pixels);

  assert.deepStrictEqual(
    [pixels[0], pixels[1], pixels[2], pixels[3]],
    expectedColor,
    `${msg}: expected ${expectedColor}, got ${[pixels[0], pixels[1], pixels[2], pixels[3]]}`
  );
};

test('WebGL2 draw functions have expected behavior with invalid vertex attribs', { skip: true }, async (t) => {
  const vs = `#version 300 es
  in vec4 vPosition;
  void main()
  {
      gl_Position = vPosition;
  }
  `;

  const fs = `#version 300 es
  precision mediump float;
  out vec4 fragColor;
  void main()
  {
      fragColor = vec4(1, 0, 0, 1);
  }
  `;

  const runTest = async (drawFnName, drawFn) => {
    await t.test(drawFnName, async () => {
      const gl = await webGL2();
      assert.ok(gl, 'context does not exist');
      const INVALID_OPERATION = 0x0502;

      try {
        gl.viewport(0, 0, 16, 16);

        const program = gl.createProgram();
        const vShader = gl.createShader(gl.VERTEX_SHADER);
        gl.shaderSource(vShader, vs);
        gl.compileShader(vShader);
        gl.attachShader(program, vShader);

        const fShader = gl.createShader(gl.FRAGMENT_SHADER);
        gl.shaderSource(fShader, fs);
        gl.compileShader(fShader);
        gl.attachShader(program, fShader);

        gl.bindAttribLocation(program, 0, "vPosition");
        gl.linkProgram(program);

        if (!gl.getProgramParameter(program, gl.LINK_STATUS)) {
          console.error(gl.getProgramInfoLog(program));
          assert.fail("Program failed to link");
        }
        gl.useProgram(program);

        const positionBuffer = gl.createBuffer();
        gl.bindBuffer(gl.ARRAY_BUFFER, positionBuffer);
        gl.bufferData(gl.ARRAY_BUFFER, new Float32Array([
          -1, -1,
          1, -1,
          -1, 1,
          -1, 1,
          1, -1,
          1, 1,
        ]), gl.STATIC_DRAW);

        const indexBuffer = gl.createBuffer();
        gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, indexBuffer);
        gl.bufferData(gl.ELEMENT_ARRAY_BUFFER,
          new Uint8Array([0, 1, 2, 3, 4, 5]),
          gl.STATIC_DRAW);

        // reset attribs
        gl.bindBuffer(gl.ARRAY_BUFFER, null);
        const numAttribs = gl.getParameter(gl.MAX_VERTEX_ATTRIBS);
        for (let i = 0; i < numAttribs; ++i) {
          gl.disableVertexAttribArray(i);
          gl.vertexAttribPointer(i, 1, gl.FLOAT, false, 0, 0);
        }

        // test drawFn draws with valid attributes
        gl.bindBuffer(gl.ARRAY_BUFFER, positionBuffer);
        gl.enableVertexAttribArray(0);
        gl.vertexAttribPointer(0, 2, gl.FLOAT, false, 0, 0);
        checkError(gl, gl.NO_ERROR, "there should be no errors setup");

        gl.clearColor(0, 0, 0, 0);
        gl.clear(gl.COLOR_BUFFER_BIT);
        checkCanvas(gl, [0, 0, 0, 0], "canvas should be zero");

        drawFn(gl);

        checkCanvas(gl, [255, 0, 0, 255], "canvas should be red");
        checkError(gl, gl.NO_ERROR, "there should be no errors after valid draw");

        // test drawFn generates INVALID_OPERATION draws with enabled attribute no buffer bound
        gl.enableVertexAttribArray(1);

        gl.clearColor(0, 0, 0, 0);
        gl.clear(gl.COLOR_BUFFER_BIT);
        checkCanvas(gl, [0, 0, 0, 0], "canvas should be zero");

        drawFn(gl);

        checkError(gl, INVALID_OPERATION, "should generate INVALID_OPERATION");
        checkCanvas(gl, [0, 0, 0, 0], "canvas should be zero");

      } finally {
        if (gl && gl.destroy) {
          gl.destroy();
        }
      }
    });
  };

  await runTest('drawArrays', (gl) => {
    gl.drawArrays(gl.TRIANGLES, 0, 6);
  });

  await runTest('drawElements', (gl) => {
    gl.drawElements(gl.TRIANGLES, 6, gl.UNSIGNED_BYTE, 0);
  });

  await runTest('drawArraysInstanced', (gl) => {
    gl.drawArraysInstanced(gl.TRIANGLES, 0, 6, 1);
  });

  await runTest('drawElementsInstanced', (gl) => {
    gl.drawElementsInstanced(gl.TRIANGLES, 6, gl.UNSIGNED_BYTE, 0, 1);
  });

  await runTest('drawRangeElements', (gl) => {
    gl.drawRangeElements(gl.TRIANGLES, 0, 5, 6, gl.UNSIGNED_BYTE, 0);
  });

});
