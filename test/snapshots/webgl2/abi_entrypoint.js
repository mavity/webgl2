// @ts-check
// ABI Test 1: Entrypoint prologue verification
// This test captures the current entry point signature and structure

import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2, getShaderWat } from '../../../index.js';

test('ABI: entrypoint has correct signature and prologue', async () => {
  const gl = await webGL2({ debug: 'rust' });

  try {
    const vs = gl.createShader(gl.VERTEX_SHADER);
    gl.shaderSource(vs, `#version 300 es
      layout(location = 0) in vec3 position;
      void main() { 
        gl_Position = vec4(position, 1.0);
      }`);
    gl.compileShader(vs);

    const fs = gl.createShader(gl.FRAGMENT_SHADER);
    gl.shaderSource(fs, `#version 300 es
      precision highp float;
      out vec4 fragColor;
      void main() { 
        fragColor = vec4(1.0, 0.0, 0.0, 1.0);
      }`);
    gl.compileShader(fs);

    const program = gl.createProgram();
    gl.attachShader(program, vs);
    gl.attachShader(program, fs);
    gl.linkProgram(program);

    const status = gl.getProgramParameter(program, gl.LINK_STATUS);
    assert.strictEqual(status, true, 'Program should link successfully');

    // Check vertex shader - full WAT snapshot
    const vsWat = getShaderWat(gl._ctxHandle, program._handle, gl.VERTEX_SHADER);
    assert.equal(vsWat, `(module
  (type (;0;) (func))
  (type (;1;) (func (param i32 i32 i32 i32 i32 i32)))
  (import "env" "memory" (memory (;0;) 10))
  (global (;0;) (mut i32) i32.const 0)
  (global (;1;) (mut i32) i32.const 0)
  (global (;2;) (mut i32) i32.const 0)
  (global (;3;) (mut i32) i32.const 0)
  (global (;4;) (mut i32) i32.const 0)
  (export "main" (func 1))
  (func (;0;) (type 0)
    (local i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32)
    global.get 2
    global.get 0
    f32.load
    f32.store
    global.get 2
    global.get 0
    i32.const 4
    i32.add
    f32.load
    f32.store offset=4
    global.get 2
    global.get 0
    i32.const 8
    i32.add
    f32.load
    f32.store offset=8
    global.get 2
    f32.const 0x1p+0 (;=1;)
    f32.store offset=12
    return
  )
  (func (;1;) (type 1) (param i32 i32 i32 i32 i32 i32)
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
    global.get 0
    global.get 0
    i32.const 0
    i32.add
    f32.load
    f32.store
    global.get 0
    global.get 0
    i32.const 4
    i32.add
    f32.load
    f32.store offset=4
    global.get 0
    global.get 0
    i32.const 8
    i32.add
    f32.load
    f32.store offset=8
    call 0
    global.get 2
    f32.load
    global.get 2
    i32.const 4
    i32.add
    f32.load
    global.get 2
    i32.const 8
    i32.add
    f32.load
    global.get 2
    i32.const 12
    i32.add
    f32.load
    local.set 39
    global.get 2
    local.get 39
    f32.store offset=12
    local.set 39
    global.get 2
    local.get 39
    f32.store offset=8
    local.set 39
    global.get 2
    local.get 39
    f32.store offset=4
    local.set 39
    global.get 2
    local.get 39
    f32.store
    return
  )
)
`);

    // Check fragment shader - full WAT snapshot
    const fsWat = getShaderWat(gl._ctxHandle, program._handle, gl.FRAGMENT_SHADER);
    assert.equal(fsWat, `(module
  (type (;0;) (func))
  (type (;1;) (func (param i32 i32 i32 i32 i32 i32)))
  (import "env" "memory" (memory (;0;) 10))
  (global (;0;) (mut i32) i32.const 0)
  (global (;1;) (mut i32) i32.const 0)
  (global (;2;) (mut i32) i32.const 0)
  (global (;3;) (mut i32) i32.const 0)
  (global (;4;) (mut i32) i32.const 0)
  (export "main" (func 1))
  (func (;0;) (type 0)
    (local i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32)
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
  (func (;1;) (type 1) (param i32 i32 i32 i32 i32 i32)
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
    call 0
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
)
`);

  } finally {
    gl.destroy();
  }
});

test('ABI: entrypoint with attributes reads from attribute pointer', async () => {
  const gl = await webGL2({ debug: 'rust' });

  try {
    const vs = gl.createShader(gl.VERTEX_SHADER);
    gl.shaderSource(vs, `#version 300 es
      layout(location = 0) in vec4 position;
      layout(location = 1) in vec2 uv;
      out vec2 v_uv;
      void main() { 
        v_uv = uv;
        gl_Position = position;
      }`);
    gl.compileShader(vs);

    const fs = gl.createShader(gl.FRAGMENT_SHADER);
    gl.shaderSource(fs, `#version 300 es
      precision highp float;
      in vec2 v_uv;
      out vec4 fragColor;
      void main() { 
        fragColor = vec4(v_uv, 0.0, 1.0);
      }`);
    gl.compileShader(fs);

    const program = gl.createProgram();
    gl.attachShader(program, vs);
    gl.attachShader(program, fs);
    gl.linkProgram(program);

    const status = gl.getProgramParameter(program, gl.LINK_STATUS);
    assert.strictEqual(status, true, 'Program should link successfully');

    // Full WAT snapshot - verifies attribute loading from global 0
    const vsWat = getShaderWat(gl._ctxHandle, program._handle, gl.VERTEX_SHADER);
    assert.equal(vsWat, `(module
  (type (;0;) (func))
  (type (;1;) (func (param i32 i32 i32 i32 i32 i32)))
  (import "env" "memory" (memory (;0;) 10))
  (global (;0;) (mut i32) i32.const 0)
  (global (;1;) (mut i32) i32.const 0)
  (global (;2;) (mut i32) i32.const 0)
  (global (;3;) (mut i32) i32.const 0)
  (global (;4;) (mut i32) i32.const 0)
  (export "main" (func 1))
  (func (;0;) (type 0)
    (local i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32)
    global.get 2
    i32.const 16
    i32.add
    global.get 0
    i32.const 64
    i32.add
    f32.load
    f32.store
    global.get 2
    i32.const 16
    i32.add
    global.get 0
    i32.const 64
    i32.add
    i32.const 4
    i32.add
    f32.load
    f32.store offset=4
    global.get 2
    global.get 0
    f32.load
    f32.store
    global.get 2
    global.get 0
    i32.const 4
    i32.add
    f32.load
    f32.store offset=4
    global.get 2
    global.get 0
    i32.const 8
    i32.add
    f32.load
    f32.store offset=8
    global.get 2
    global.get 0
    i32.const 12
    i32.add
    f32.load
    f32.store offset=12
    return
  )
  (func (;1;) (type 1) (param i32 i32 i32 i32 i32 i32)
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
    global.get 0
    global.get 0
    i32.const 0
    i32.add
    f32.load
    f32.store
    global.get 0
    global.get 0
    i32.const 4
    i32.add
    f32.load
    f32.store offset=4
    global.get 0
    global.get 0
    i32.const 8
    i32.add
    f32.load
    f32.store offset=8
    global.get 0
    global.get 0
    i32.const 12
    i32.add
    f32.load
    f32.store offset=12
    global.get 0
    i32.const 64
    i32.add
    global.get 0
    i32.const 64
    i32.add
    f32.load
    f32.store
    global.get 0
    i32.const 64
    i32.add
    global.get 0
    i32.const 68
    i32.add
    f32.load
    f32.store offset=4
    call 0
    global.get 2
    i32.const 16
    i32.add
    f32.load
    global.get 2
    i32.const 16
    i32.add
    i32.const 4
    i32.add
    f32.load
    global.get 2
    f32.load
    global.get 2
    i32.const 4
    i32.add
    f32.load
    global.get 2
    i32.const 8
    i32.add
    f32.load
    global.get 2
    i32.const 12
    i32.add
    f32.load
    local.set 39
    global.get 2
    local.get 39
    f32.store offset=12
    local.set 39
    global.get 2
    local.get 39
    f32.store offset=8
    local.set 39
    global.get 2
    local.get 39
    f32.store offset=4
    local.set 39
    global.get 2
    local.get 39
    f32.store
    local.set 39
    global.get 2
    local.get 39
    f32.store offset=20
    local.set 39
    global.get 2
    local.get 39
    f32.store offset=16
    return
  )
)
`);

  } finally {
    gl.destroy();
  }
});
