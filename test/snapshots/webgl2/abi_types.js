// @ts-check
// ABI Test 3: Parameter type validation and error cases
// This test verifies current behavior for edge cases and type limits

import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2, getShaderWat } from '../../../index.js';

test('ABI: void function with no parameters', async () => {
  const gl = await webGL2({ debug: 'rust' });

  try {
    const fs = gl.createShader(gl.FRAGMENT_SHADER);
    gl.shaderSource(fs, `#version 300 es
      precision highp float;
      out vec4 fragColor;
      
      void doNothing() {
        // Empty function
      }
      
      void main() {
        doNothing();
        fragColor = vec4(1.0, 0.0, 0.0, 1.0);
      }`);
    gl.compileShader(fs);

    const vs = gl.createShader(gl.VERTEX_SHADER);
    gl.shaderSource(vs, `#version 300 es
      void main() { gl_Position = vec4(0); }`);
    gl.compileShader(vs);

    const program = gl.createProgram();
    gl.attachShader(program, vs);
    gl.attachShader(program, fs);
    gl.linkProgram(program);

    const status = gl.getProgramParameter(program, gl.LINK_STATUS);
    assert.strictEqual(status, true, 'Program with void function should link successfully');

    const fsWat = getShaderWat(gl._ctxHandle, program._handle, gl.FRAGMENT_SHADER);

    // Full WAT snapshot for void function case
    assert.equal(fsWat, `(module
  (type (;0;) (func))
  (type (;1;) (func))
  (type (;2;) (func (param i32 i32 i32 i32 i32 i32)))
  (import "env" "memory" (memory (;0;) 10))
  (global (;0;) (mut i32) i32.const 0)
  (global (;1;) (mut i32) i32.const 0)
  (global (;2;) (mut i32) i32.const 0)
  (global (;3;) (mut i32) i32.const 0)
  (global (;4;) (mut i32) i32.const 0)
  (export "main" (func 2))
  (func (;0;) (type 0)
    (local i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32)
    return
  )
  (func (;1;) (type 1)
    (local i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32)
    call 0
    global.get 3
    f32.const 0x1p+0 (;=1;)
    f32.store
    global.get 3
    f32.const 0x0p+0 (;=0;)
    f32.store offset=4
    global.get 3
    f32.const 0x0p+0 (;=0;)
    f32.store offset=8
    global.get 3
    f32.const 0x1p+0 (;=1;)
    f32.store offset=12
    return
  )
  (func (;2;) (type 2) (param i32 i32 i32 i32 i32 i32)
    (local i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32)
    local.get 1
    global.set 0
    local.get 2
    global.set 1
    local.get 3
    global.set 2
    local.get 4
    global.set 3
    local.get 5
    global.set 4
    call 1
    global.get 3
    f32.load
    global.get 3
    i32.const 4
    i32.add
    f32.load
    global.get 3
    i32.const 8
    i32.add
    f32.load
    global.get 3
    i32.const 12
    i32.add
    f32.load
    local.set 39
    global.get 3
    local.get 39
    f32.store offset=12
    local.set 39
    global.get 3
    local.get 39
    f32.store offset=8
    local.set 39
    global.get 3
    local.get 39
    f32.store offset=4
    local.set 39
    global.get 3
    local.get 39
    f32.store
    return
  )
`)

  } finally {
    gl.destroy();
  }
});

test('ABI: multiple scalar parameters in order', async () => {
  const gl = await webGL2({ debug: 'rust' });

  try {
    const fs = gl.createShader(gl.FRAGMENT_SHADER);
    gl.shaderSource(fs, `#version 300 es
      precision highp float;
      out vec4 fragColor;
      
      float compute(float a, float b, float c) {
        return a + b + c;
      }
      
      void main() {
        float result = compute(1.0, 2.0, 3.0);
        fragColor = vec4(result, 0.0, 0.0, 1.0);
      }`);
    gl.compileShader(fs);

    const vs = gl.createShader(gl.VERTEX_SHADER);
    gl.shaderSource(vs, `#version 300 es
      void main() { gl_Position = vec4(0); }`);
    gl.compileShader(vs);

    const program = gl.createProgram();
    gl.attachShader(program, vs);
    gl.attachShader(program, fs);
    gl.linkProgram(program);

    const status = gl.getProgramParameter(program, gl.LINK_STATUS);
    assert.strictEqual(status, true, 'Program should link successfully');

    const fsWat = getShaderWat(gl._ctxHandle, program._handle, gl.FRAGMENT_SHADER);

    // Full WAT snapshot for compute(a,b,c)
    assert.equal(fsWat, `(module
  (type (;0;) (func (param f32 f32 f32) (result f32)))
  (type (;1;) (func))
  (type (;2;) (func (param i32 i32 i32 i32 i32 i32)))
  (import "env" "memory" (memory (;0;) 10))
  (global (;0;) (mut i32) i32.const 0)
  (global (;1;) (mut i32) i32.const 0)
  (global (;2;) (mut i32) i32.const 0)
  (global (;3;) (mut i32) i32.const 0)
  (global (;4;) (mut i32) i32.const 0)
  (export "main" (func 2))
  (func (;0;) (type 0) (param f32 f32 f32) (result f32)
    (local i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32)
    global.get 3
    local.get 0
    f32.store
    global.get 3
    i32.const 4
    i32.add
    local.get 1
    f32.store
    global.get 3
    i32.const 8
    i32.add
    local.get 2
    f32.store
    global.get 3
    f32.load
    global.get 3
    i32.const 4
    i32.add
    f32.load
    f32.add
    global.get 3
    i32.const 8
    i32.add
    f32.load
    f32.add
    return
  )
  (func (;1;) (type 1)
    (local f32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32)
    f32.const 0x1p+0 (;=1;)
    f32.const 0x1p+1 (;=2;)
    f32.const 0x1.8p+1 (;=3;)
    call 0
    local.set 0
    global.get 3
    local.get 0
    f32.store
    global.get 3
    global.get 3
    f32.load
    f32.store
    global.get 3
    f32.const 0x0p+0 (;=0;)
    f32.store offset=4
    global.get 3
    f32.const 0x0p+0 (;=0;)
    f32.store offset=8
    global.get 3
    f32.const 0x1p+0 (;=1;)
    f32.store offset=12
    return
  )
  (func (;2;) (type 2) (param i32 i32 i32 i32 i32 i32)
    (local i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32)
    local.get 1
    global.set 0
    local.get 2
    global.set 1
    local.get 3
    global.set 2
    local.get 4
    global.set 3
    local.get 5
    global.set 4
    call 1
    global.get 3
    f32.load
    global.get 3
    i32.const 4
    i32.add
    f32.load
    global.get 3
    i32.const 8
    i32.add
    f32.load
    global.get 3
    i32.const 12
    i32.add
    f32.load
    local.set 39
    global.get 3
    local.get 39
    f32.store offset=12
    local.set 39
    global.get 3
    local.get 39
    f32.store offset=8
    local.set 39
    global.get 3
    local.get 39
    f32.store offset=4
    local.set 39
    global.get 3
    local.get 39
    f32.store
    return
  )
`)

  } finally {
    gl.destroy();
  }
});

test('ABI: bool parameter handled as i32', async () => {
  const gl = await webGL2({ debug: 'rust' });

  try {
    const fs = gl.createShader(gl.FRAGMENT_SHADER);
    gl.shaderSource(fs, `#version 300 es
      precision highp float;
      out vec4 fragColor;
      
      float conditional(bool flag) {
        return flag ? 1.0 : 0.0;
      }
      
      void main() {
        float result = conditional(true);
        fragColor = vec4(result, 0.0, 0.0, 1.0);
      }`);
    gl.compileShader(fs);

    const vs = gl.createShader(gl.VERTEX_SHADER);
    gl.shaderSource(vs, `#version 300 es
      void main() { gl_Position = vec4(0); }`);
    gl.compileShader(vs);

    const program = gl.createProgram();
    gl.attachShader(program, vs);
    gl.attachShader(program, fs);
    gl.linkProgram(program);

    const status = gl.getProgramParameter(program, gl.LINK_STATUS);
    assert.strictEqual(status, true, 'Program with bool parameter should link successfully');

    const fsWat = getShaderWat(gl._ctxHandle, program._handle, gl.FRAGMENT_SHADER);

    // Full WAT snapshot for bool parameter case
    assert.equal(fsWat, `(module
  (type (;0;) (func (param i32) (result f32)))
  (type (;1;) (func))
  (type (;2;) (func (param i32 i32 i32 i32 i32 i32)))
  (import "env" "memory" (memory (;0;) 10))
  (global (;0;) (mut i32) i32.const 0)
  (global (;1;) (mut i32) i32.const 0)
  (global (;2;) (mut i32) i32.const 0)
  (global (;3;) (mut i32) i32.const 0)
  (global (;4;) (mut i32) i32.const 0)
  (export "main" (func 2))
  (func (;0;) (type 0) (param i32) (result f32)
    (local i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32)
    global.get 3
    local.get 0
    i32.store
    global.get 3
    i32.load
    if ;; label = @1
      global.get 3
      i32.const 1
      i32.add
      f32.const 0x1p+0 (;=1;)
      f32.store
    else
      global.get 3
      i32.const 1
      i32.add
      f32.const 0x0p+0 (;=0;)
      f32.store
    end
    global.get 3
    i32.const 1
    i32.add
    f32.load
    return
  )
  (func (;1;) (type 1)
    (local f32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32)
    i32.const 1
    call 0
    local.set 0
    global.get 3
    local.get 0
    f32.store
    global.get 3
    global.get 3
    f32.load
    f32.store
    global.get 3
    f32.const 0x0p+0 (;=0;)
    f32.store offset=4
    global.get 3
    f32.const 0x0p+0 (;=0;)
    f32.store offset=8
    global.get 3
    f32.const 0x1p+0 (;=1;)
    f32.store offset=12
    return
  )
`)

  } finally {
    gl.destroy();
  }
});

test('ABI: integer parameter handling', async () => {
  const gl = await webGL2({ debug: 'rust' });

  try {
    const fs = gl.createShader(gl.FRAGMENT_SHADER);
    gl.shaderSource(fs, `#version 300 es
      precision highp float;
      out vec4 fragColor;
      
      float useInt(int val) {
        return float(val);
      }
      
      void main() {
        float result = useInt(42);
        fragColor = vec4(result / 255.0, 0.0, 0.0, 1.0);
      }`);
    gl.compileShader(fs);

    const vs = gl.createShader(gl.VERTEX_SHADER);
    gl.shaderSource(vs, `#version 300 es
      void main() { gl_Position = vec4(0); }`);
    gl.compileShader(vs);

    const program = gl.createProgram();
    gl.attachShader(program, vs);
    gl.attachShader(program, fs);
    gl.linkProgram(program);

    const status = gl.getProgramParameter(program, gl.LINK_STATUS);
    assert.strictEqual(status, true, 'Program with int parameter should link successfully');

    const fsWat = getShaderWat(gl._ctxHandle, program._handle, gl.FRAGMENT_SHADER);

    // Int should be represented as i32
    assert.ok(fsWat.includes('(func (param i32)') || fsWat.includes('(func (param i32) (result f32)'),
      'Int parameter should be i32 in WASM');

  } finally {
    gl.destroy();
  }
});

