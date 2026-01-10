// @ts-check
// ABI Test 2: Single internal function call with scalar parameters
// This test captures the current behavior of calling internal functions with simple scalar params

import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2, getShaderWat } from '../../../index.js';

test('ABI: single internal function with scalar param (flattened)', async () => {
  const gl = await webGL2({ debug: 'rust' });

  try {
    const fs = gl.createShader(gl.FRAGMENT_SHADER);
    gl.shaderSource(fs, `#version 300 es
      precision highp float;
      out vec4 fragColor;
      
      void helper(float val) {
        // Simple function that does nothing with the value
        val = val + 1.0;
      }
      
      void main() {
        helper(3.0);
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
    assert.strictEqual(status, true, 'Program should link successfully');

    const fsWat = getShaderWat(gl._ctxHandle, program._handle, gl.FRAGMENT_SHADER);

    // Full WAT snapshot for helper(float)
    assert.equal(fsWat, `(module
  (type (;0;) (func (param f32)))
  (type (;1;) (func))
  (type (;2;) (func (param i32 i32 i32 i32 i32 i32)))
  (import "env" "memory" (memory (;0;) 10))
  (global (;0;) (mut i32) i32.const 0)
  (global (;1;) (mut i32) i32.const 0)
  (global (;2;) (mut i32) i32.const 0)
  (global (;3;) (mut i32) i32.const 0)
  (global (;4;) (mut i32) i32.const 0)
  (export "main" (func 2))
  (func (;0;) (type 0) (param f32)
    (local i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32)
    global.get 3
    local.get 0
    f32.store
    global.get 3
    global.get 3
    f32.load
    f32.const 0x1p+0 (;=1;)
    f32.add
    f32.store
    return
  )
  (func (;1;) (type 1)
    (local i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32)
    f32.const 0x1.8p+1 (;=3;)
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
)`);

  } finally {
    gl.destroy();
  }
});

test('ABI: internal function with vec2 param (flattened)', async () => {
  const gl = await webGL2({ debug: 'rust' });

  try {
    const fs = gl.createShader(gl.FRAGMENT_SHADER);
    gl.shaderSource(fs, `#version 300 es
      precision highp float;
      out vec4 fragColor;
      
      float helper(vec2 val) {
        return val.x + val.y;
      }
      
      void main() {
        float result = helper(vec2(1.0, 2.0));
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

    // Full WAT snapshot for helper(vec2) returning float
    assert.equal(fsWat, `(module
  (type (;0;) (func (param f32 f32) (result f32)))
  (type (;1;) (func))
  (type (;2;) (func (param i32 i32 i32 i32 i32 i32)))
  (import "env" "memory" (memory (;0;) 10))
  (global (;0;) (mut i32) i32.const 0)
  (global (;1;) (mut i32) i32.const 0)
  (global (;2;) (mut i32) i32.const 0)
  (global (;3;) (mut i32) i32.const 0)
  (global (;4;) (mut i32) i32.const 0)
  (export "main" (func 2))
  (func (;0;) (type 0) (param f32 f32) (result f32)
    (local i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32)
    global.get 3
    local.get 0
    f32.store
    global.get 3
    local.get 1
    f32.store offset=4
    global.get 3
    f32.load
    global.get 3
    i32.const 4
    i32.add
    f32.load
    f32.add
    return
  )
  (func (;1;) (type 1)
    (local f32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32)
    f32.const 0x1p+0 (;=1;)
    f32.const 0x1p+1 (;=2;)
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
)`);

  } finally {
    gl.destroy();
  }
});

test('ABI: internal function with scalar return value', async () => {
  const gl = await webGL2({ debug: 'rust' });

  try {
    const fs = gl.createShader(gl.FRAGMENT_SHADER);
    gl.shaderSource(fs, `#version 300 es
      precision highp float;
      out vec4 fragColor;
      
      float compute() {
        return 0.5;
      }
      
      void main() {
        float val = compute();
        fragColor = vec4(val, val, val, 1.0);
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

    // Full WAT snapshot for compute() returning f32
    assert.equal(fsWat, `(module
  (type (;0;) (func (result f32)))
  (type (;1;) (func))
  (type (;2;) (func (param i32 i32 i32 i32 i32 i32)))
  (import "env" "memory" (memory (;0;) 10))
  (global (;0;) (mut i32) i32.const 0)
  (global (;1;) (mut i32) i32.const 0)
  (global (;2;) (mut i32) i32.const 0)
  (global (;3;) (mut i32) i32.const 0)
  (global (;4;) (mut i32) i32.const 0)
  (export "main" (func 2))
  (func (;0;) (type 0) (result f32)
    (local i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32)
    f32.const 0x1p-1 (;=0.5;)
    return
  )
  (func (;1;) (type 1)
    (local f32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32)
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
    global.get 3
    f32.load
    f32.store offset=4
    global.get 3
    global.get 3
    f32.load
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
)`);

  } finally {
    gl.destroy();
  }
});

test('ABI: internal function with vec3 return (flattened)', async () => {
  const gl = await webGL2({ debug: 'rust' });

  try {
    const fs = gl.createShader(gl.FRAGMENT_SHADER);
    gl.shaderSource(fs, `#version 300 es
      precision highp float;
      out vec4 fragColor;
      
      vec3 getColor() {
        return vec3(1.0, 0.0, 0.0);
      }
      
      void main() {
        vec3 color = getColor();
        fragColor = vec4(color, 1.0);
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

    // Full WAT snapshot for getColor() returning vec3
    assert.equal(fsWat, `(module
  (type (;0;) (func (result f32 f32 f32)))
  (type (;1;) (func))
  (type (;2;) (func (param i32 i32 i32 i32 i32 i32)))
  (import "env" "memory" (memory (;0;) 10))
  (global (;0;) (mut i32) i32.const 0)
  (global (;1;) (mut i32) i32.const 0)
  (global (;2;) (mut i32) i32.const 0)
  (global (;3;) (mut i32) i32.const 0)
  (global (;4;) (mut i32) i32.const 0)
  (export "main" (func 2))
  (func (;0;) (type 0) (result f32 f32 f32)
    (local i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32)
    f32.const 0x1p+0 (;=1;)
    f32.const 0x0p+0 (;=0;)
    f32.const 0x0p+0 (;=0;)
    return
  )
  (func (;1;) (type 1)
    (local f32 f32 f32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32)
    call 0
    local.set 2
    local.set 1
    local.set 0
    global.get 3
    local.get 0
    f32.store
    global.get 3
    local.get 1
    f32.store offset=4
    global.get 3
    local.get 2
    f32.store offset=8
    global.get 3
    global.get 3
    f32.load
    f32.store
    global.get 3
    global.get 3
    i32.const 4
    i32.add
    f32.load
    f32.store offset=4
    global.get 3
    global.get 3
    i32.const 8
    i32.add
    f32.load
    f32.store offset=8
    global.get 3
    f32.const 0x1p+0 (;=1;)
    f32.store offset=12
    return
  )
)`);

  } finally {
    gl.destroy();
  }
});
