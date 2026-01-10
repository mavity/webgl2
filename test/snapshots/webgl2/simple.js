// @ts-check

import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2, getShaderWat } from '../../../index.js';

test('basic vertex shader WAT', async () => {
  const gl = await webGL2({ debug: 'rust' });

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

    // Get WAT text for vertex shader - use program._handle to get the numeric handle
    const wat = getShaderWat(gl._ctxHandle, program._handle, gl.VERTEX_SHADER);

    // WAT files start with (module
    assert.equal(
      wat, `(module
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
    f32.const 0x0p+0 (;=0;)
    f32.store
    global.get 2
    f32.const 0x0p+0 (;=0;)
    f32.store offset=4
    global.get 2
    f32.const 0x0p+0 (;=0;)
    f32.store offset=8
    global.get 2
    f32.const 0x0p+0 (;=0;)
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
`
    );
  } finally {
    gl.destroy();
  }
});

test('basic fragment shader WAT', async () => {
  const gl = await webGL2({ debug: 'rust' });

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

    // Get WAT text for fragment shader - use program._handle to get the numeric handle
    const wat = getShaderWat(gl._ctxHandle, program._handle, gl.FRAGMENT_SHADER);

    // WAT files start with (module
    assert.equal(
      wat, `(module
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
    f32.const 0x0p+0 (;=0;)
    f32.store
    global.get 2
    f32.const 0x0p+0 (;=0;)
    f32.store offset=4
    global.get 2
    f32.const 0x0p+0 (;=0;)
    f32.store offset=8
    global.get 2
    f32.const 0x0p+0 (;=0;)
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
`
    );
  } finally {
    gl.destroy();
  }
});