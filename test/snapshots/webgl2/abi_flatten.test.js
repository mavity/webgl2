// @ts-check
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
      void helper(float val) { val = val + 1.0; }
      void main() { helper(3.0); fragColor = vec4(1.0); }`);
    gl.compileShader(fs);

    const vs = gl.createShader(gl.VERTEX_SHADER);
    gl.shaderSource(vs, `#version 300 es
      void main() { gl_Position = vec4(0); }`);
    gl.compileShader(vs);

    const program = gl.createProgram();
    gl.attachShader(program, vs);
    gl.attachShader(program, fs);
    gl.linkProgram(program);

    const fsWat = getShaderWat(gl._ctxHandle, program._handle, gl.FRAGMENT_SHADER);
    assert.strictEqual(fsWat, `(module
  (type (;0;) (func (param f32) (result f32)))
  (type (;1;) (func (param f32) (result f32)))
  (type (;2;) (func (param f32) (result f32)))
  (type (;3;) (func (param f32) (result f32)))
  (type (;4;) (func (param f32) (result f32)))
  (type (;5;) (func (param f32) (result f32)))
  (type (;6;) (func (param f32 f32) (result f32)))
  (type (;7;) (func (param f32) (result f32)))
  (type (;8;) (func (param f32) (result f32)))
  (type (;9;) (func (param f32) (result f32)))
  (type (;10;) (func (param f32) (result f32)))
  (type (;11;) (func (param f32 f32) (result f32)))
  (type (;12;) (func (param f32) (result f32)))
  (type (;13;) (func (param f32) (result f32)))
  (type (;14;) (func (param f32) (result f32)))
  (type (;15;) (func (param f32) (result f32)))
  (type (;16;) (func (param f32) (result f32)))
  (type (;17;) (func (param f32) (result f32)))
  (type (;18;) (func (param f32)))
  (type (;19;) (func))
  (type (;20;) (func (param i32 i32 i32 i32 i32 i32 i32 i32 i32)))
  (import "env" "memory" (memory (;0;) 100))
  (import "env" "gl_sin" (func (;0;) (type 0)))
  (import "env" "gl_cos" (func (;1;) (type 1)))
  (import "env" "gl_tan" (func (;2;) (type 2)))
  (import "env" "gl_asin" (func (;3;) (type 3)))
  (import "env" "gl_acos" (func (;4;) (type 4)))
  (import "env" "gl_atan" (func (;5;) (type 5)))
  (import "env" "gl_atan2" (func (;6;) (type 6)))
  (import "env" "gl_exp" (func (;7;) (type 7)))
  (import "env" "gl_exp2" (func (;8;) (type 8)))
  (import "env" "gl_log" (func (;9;) (type 9)))
  (import "env" "gl_log2" (func (;10;) (type 10)))
  (import "env" "gl_pow" (func (;11;) (type 11)))
  (import "env" "gl_sinh" (func (;12;) (type 12)))
  (import "env" "gl_cosh" (func (;13;) (type 13)))
  (import "env" "gl_tanh" (func (;14;) (type 14)))
  (import "env" "gl_asinh" (func (;15;) (type 15)))
  (import "env" "gl_acosh" (func (;16;) (type 16)))
  (import "env" "gl_atanh" (func (;17;) (type 17)))
  (global (;0;) (mut i32) i32.const 0)
  (global (;1;) (mut i32) i32.const 0)
  (global (;2;) (mut i32) i32.const 0)
  (global (;3;) (mut i32) i32.const 0)
  (global (;4;) (mut i32) i32.const 0)
  (global (;5;) (mut i32) i32.const 0)
  (export "main" (func 20))
  (func (;18;) (type 18) (param f32)
    (local i32 f32 i32)
    global.get 3
    i32.const 16
    i32.add
    local.get 0
    f32.store
    global.get 3
    i32.const 16
    i32.add
    global.get 3
    i32.const 16
    i32.add
    f32.load
    f32.const 0x1p+0 (;=1;)
    f32.add
    f32.store
    return
  )
  (func (;19;) (type 19)
    (local i32 f32 i32)
    f32.const 0x1.8p+1 (;=3;)
    call 18
    global.get 3
    f32.const 0x1p+0 (;=1;)
    f32.store
    global.get 3
    f32.const 0x1p+0 (;=1;)
    f32.store offset=4
    global.get 3
    f32.const 0x1p+0 (;=1;)
    f32.store offset=8
    global.get 3
    f32.const 0x1p+0 (;=1;)
    f32.store offset=12
    return
  )
  (func (;20;) (type 20) (param i32 i32 i32 i32 i32 i32 i32 i32 i32)
    (local i32 f32 i32)
    local.get 3
    global.set 0
    local.get 4
    global.set 1
    local.get 5
    global.set 2
    local.get 6
    global.set 3
    local.get 7
    global.set 4
    local.get 8
    global.set 5
    call 19
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
    local.set 10
    global.get 3
    local.get 10
    f32.store offset=12
    local.set 10
    global.get 3
    local.get 10
    f32.store offset=8
    local.set 10
    global.get 3
    local.get 10
    f32.store offset=4
    local.set 10
    global.get 3
    local.get 10
    f32.store
    return
  )
)
`);
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
      float helper(vec2 val) { return val.x + val.y; }
      void main() { float result = helper(vec2(1.0,2.0)); fragColor = vec4(result,0.0,0.0,1.0);} `);
    gl.compileShader(fs);

    const vs = gl.createShader(gl.VERTEX_SHADER);
    gl.shaderSource(vs, `#version 300 es
      void main() { gl_Position = vec4(0); }`);
    gl.compileShader(vs);

    const program = gl.createProgram();
    gl.attachShader(program, vs);
    gl.attachShader(program, fs);
    gl.linkProgram(program);

    const fsWat = getShaderWat(gl._ctxHandle, program._handle, gl.FRAGMENT_SHADER);
    assert.strictEqual(fsWat, `(module
  (type (;0;) (func (param f32) (result f32)))
  (type (;1;) (func (param f32) (result f32)))
  (type (;2;) (func (param f32) (result f32)))
  (type (;3;) (func (param f32) (result f32)))
  (type (;4;) (func (param f32) (result f32)))
  (type (;5;) (func (param f32) (result f32)))
  (type (;6;) (func (param f32 f32) (result f32)))
  (type (;7;) (func (param f32) (result f32)))
  (type (;8;) (func (param f32) (result f32)))
  (type (;9;) (func (param f32) (result f32)))
  (type (;10;) (func (param f32) (result f32)))
  (type (;11;) (func (param f32 f32) (result f32)))
  (type (;12;) (func (param f32) (result f32)))
  (type (;13;) (func (param f32) (result f32)))
  (type (;14;) (func (param f32) (result f32)))
  (type (;15;) (func (param f32) (result f32)))
  (type (;16;) (func (param f32) (result f32)))
  (type (;17;) (func (param f32) (result f32)))
  (type (;18;) (func (param f32 f32) (result f32)))
  (type (;19;) (func))
  (type (;20;) (func (param i32 i32 i32 i32 i32 i32 i32 i32 i32)))
  (import "env" "memory" (memory (;0;) 100))
  (import "env" "gl_sin" (func (;0;) (type 0)))
  (import "env" "gl_cos" (func (;1;) (type 1)))
  (import "env" "gl_tan" (func (;2;) (type 2)))
  (import "env" "gl_asin" (func (;3;) (type 3)))
  (import "env" "gl_acos" (func (;4;) (type 4)))
  (import "env" "gl_atan" (func (;5;) (type 5)))
  (import "env" "gl_atan2" (func (;6;) (type 6)))
  (import "env" "gl_exp" (func (;7;) (type 7)))
  (import "env" "gl_exp2" (func (;8;) (type 8)))
  (import "env" "gl_log" (func (;9;) (type 9)))
  (import "env" "gl_log2" (func (;10;) (type 10)))
  (import "env" "gl_pow" (func (;11;) (type 11)))
  (import "env" "gl_sinh" (func (;12;) (type 12)))
  (import "env" "gl_cosh" (func (;13;) (type 13)))
  (import "env" "gl_tanh" (func (;14;) (type 14)))
  (import "env" "gl_asinh" (func (;15;) (type 15)))
  (import "env" "gl_acosh" (func (;16;) (type 16)))
  (import "env" "gl_atanh" (func (;17;) (type 17)))
  (global (;0;) (mut i32) i32.const 0)
  (global (;1;) (mut i32) i32.const 0)
  (global (;2;) (mut i32) i32.const 0)
  (global (;3;) (mut i32) i32.const 0)
  (global (;4;) (mut i32) i32.const 0)
  (global (;5;) (mut i32) i32.const 0)
  (export "main" (func 20))
  (func (;18;) (type 18) (param f32 f32) (result f32)
    (local i32 f32 i32)
    global.get 3
    i32.const 16
    i32.add
    local.get 0
    f32.store
    global.get 3
    i32.const 16
    i32.add
    local.get 1
    f32.store offset=4
    global.get 3
    i32.const 16
    i32.add
    f32.load
    global.get 3
    i32.const 16
    i32.add
    i32.const 4
    i32.add
    f32.load
    f32.add
    return
  )
  (func (;19;) (type 19)
    (local f32 i32 f32 i32)
    f32.const 0x1p+0 (;=1;)
    f32.const 0x1p+1 (;=2;)
    call 18
    local.set 0
    global.get 3
    i32.const 16
    i32.add
    local.get 0
    f32.store
    global.get 3
    global.get 3
    i32.const 16
    i32.add
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
  (func (;20;) (type 20) (param i32 i32 i32 i32 i32 i32 i32 i32 i32)
    (local i32 f32 i32)
    local.get 3
    global.set 0
    local.get 4
    global.set 1
    local.get 5
    global.set 2
    local.get 6
    global.set 3
    local.get 7
    global.set 4
    local.get 8
    global.set 5
    call 19
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
    local.set 10
    global.get 3
    local.get 10
    f32.store offset=12
    local.set 10
    global.get 3
    local.get 10
    f32.store offset=8
    local.set 10
    global.get 3
    local.get 10
    f32.store offset=4
    local.set 10
    global.get 3
    local.get 10
    f32.store
    return
  )
)
`);
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
      float compute() { return 0.5; }
      void main() { float val = compute(); fragColor = vec4(val,val,val,1.0); }`);
    gl.compileShader(fs);

    const vs = gl.createShader(gl.VERTEX_SHADER);
    gl.shaderSource(vs, `#version 300 es
      void main() { gl_Position = vec4(0); }`);
    gl.compileShader(vs);

    const program = gl.createProgram();
    gl.attachShader(program, vs);
    gl.attachShader(program, fs);
    gl.linkProgram(program);

    const fsWat = getShaderWat(gl._ctxHandle, program._handle, gl.FRAGMENT_SHADER);
    assert.strictEqual(fsWat, `(module
  (type (;0;) (func (param f32) (result f32)))
  (type (;1;) (func (param f32) (result f32)))
  (type (;2;) (func (param f32) (result f32)))
  (type (;3;) (func (param f32) (result f32)))
  (type (;4;) (func (param f32) (result f32)))
  (type (;5;) (func (param f32) (result f32)))
  (type (;6;) (func (param f32 f32) (result f32)))
  (type (;7;) (func (param f32) (result f32)))
  (type (;8;) (func (param f32) (result f32)))
  (type (;9;) (func (param f32) (result f32)))
  (type (;10;) (func (param f32) (result f32)))
  (type (;11;) (func (param f32 f32) (result f32)))
  (type (;12;) (func (param f32) (result f32)))
  (type (;13;) (func (param f32) (result f32)))
  (type (;14;) (func (param f32) (result f32)))
  (type (;15;) (func (param f32) (result f32)))
  (type (;16;) (func (param f32) (result f32)))
  (type (;17;) (func (param f32) (result f32)))
  (type (;18;) (func (result f32)))
  (type (;19;) (func))
  (type (;20;) (func (param i32 i32 i32 i32 i32 i32 i32 i32 i32)))
  (import "env" "memory" (memory (;0;) 100))
  (import "env" "gl_sin" (func (;0;) (type 0)))
  (import "env" "gl_cos" (func (;1;) (type 1)))
  (import "env" "gl_tan" (func (;2;) (type 2)))
  (import "env" "gl_asin" (func (;3;) (type 3)))
  (import "env" "gl_acos" (func (;4;) (type 4)))
  (import "env" "gl_atan" (func (;5;) (type 5)))
  (import "env" "gl_atan2" (func (;6;) (type 6)))
  (import "env" "gl_exp" (func (;7;) (type 7)))
  (import "env" "gl_exp2" (func (;8;) (type 8)))
  (import "env" "gl_log" (func (;9;) (type 9)))
  (import "env" "gl_log2" (func (;10;) (type 10)))
  (import "env" "gl_pow" (func (;11;) (type 11)))
  (import "env" "gl_sinh" (func (;12;) (type 12)))
  (import "env" "gl_cosh" (func (;13;) (type 13)))
  (import "env" "gl_tanh" (func (;14;) (type 14)))
  (import "env" "gl_asinh" (func (;15;) (type 15)))
  (import "env" "gl_acosh" (func (;16;) (type 16)))
  (import "env" "gl_atanh" (func (;17;) (type 17)))
  (global (;0;) (mut i32) i32.const 0)
  (global (;1;) (mut i32) i32.const 0)
  (global (;2;) (mut i32) i32.const 0)
  (global (;3;) (mut i32) i32.const 0)
  (global (;4;) (mut i32) i32.const 0)
  (global (;5;) (mut i32) i32.const 0)
  (export "main" (func 20))
  (func (;18;) (type 18) (result f32)
    (local i32 f32 i32)
    f32.const 0x1p-1 (;=0.5;)
    return
  )
  (func (;19;) (type 19)
    (local f32 i32 f32 i32)
    call 18
    local.set 0
    global.get 3
    i32.const 16
    i32.add
    local.get 0
    f32.store
    global.get 3
    global.get 3
    i32.const 16
    i32.add
    f32.load
    f32.store
    global.get 3
    global.get 3
    i32.const 16
    i32.add
    f32.load
    f32.store offset=4
    global.get 3
    global.get 3
    i32.const 16
    i32.add
    f32.load
    f32.store offset=8
    global.get 3
    f32.const 0x1p+0 (;=1;)
    f32.store offset=12
    return
  )
  (func (;20;) (type 20) (param i32 i32 i32 i32 i32 i32 i32 i32 i32)
    (local i32 f32 i32)
    local.get 3
    global.set 0
    local.get 4
    global.set 1
    local.get 5
    global.set 2
    local.get 6
    global.set 3
    local.get 7
    global.set 4
    local.get 8
    global.set 5
    call 19
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
    local.set 10
    global.get 3
    local.get 10
    f32.store offset=12
    local.set 10
    global.get 3
    local.get 10
    f32.store offset=8
    local.set 10
    global.get 3
    local.get 10
    f32.store offset=4
    local.set 10
    global.get 3
    local.get 10
    f32.store
    return
  )
)
`);
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
      vec3 makeVec3() { return vec3(1.0, 0.0, 0.0); }
      void main() { vec3 v = makeVec3(); fragColor = vec4(v,1.0);} `);
    gl.compileShader(fs);

    const vs = gl.createShader(gl.VERTEX_SHADER);
    gl.shaderSource(vs, `#version 300 es
      void main() { gl_Position = vec4(0); }`);
    gl.compileShader(vs);

    const program = gl.createProgram();
    gl.attachShader(program, vs);
    gl.attachShader(program, fs);
    gl.linkProgram(program);

    const fsWat = getShaderWat(gl._ctxHandle, program._handle, gl.FRAGMENT_SHADER);
    assert.strictEqual(fsWat, `(module
  (type (;0;) (func (param f32) (result f32)))
  (type (;1;) (func (param f32) (result f32)))
  (type (;2;) (func (param f32) (result f32)))
  (type (;3;) (func (param f32) (result f32)))
  (type (;4;) (func (param f32) (result f32)))
  (type (;5;) (func (param f32) (result f32)))
  (type (;6;) (func (param f32 f32) (result f32)))
  (type (;7;) (func (param f32) (result f32)))
  (type (;8;) (func (param f32) (result f32)))
  (type (;9;) (func (param f32) (result f32)))
  (type (;10;) (func (param f32) (result f32)))
  (type (;11;) (func (param f32 f32) (result f32)))
  (type (;12;) (func (param f32) (result f32)))
  (type (;13;) (func (param f32) (result f32)))
  (type (;14;) (func (param f32) (result f32)))
  (type (;15;) (func (param f32) (result f32)))
  (type (;16;) (func (param f32) (result f32)))
  (type (;17;) (func (param f32) (result f32)))
  (type (;18;) (func (result f32 f32 f32)))
  (type (;19;) (func))
  (type (;20;) (func (param i32 i32 i32 i32 i32 i32 i32 i32 i32)))
  (import "env" "memory" (memory (;0;) 100))
  (import "env" "gl_sin" (func (;0;) (type 0)))
  (import "env" "gl_cos" (func (;1;) (type 1)))
  (import "env" "gl_tan" (func (;2;) (type 2)))
  (import "env" "gl_asin" (func (;3;) (type 3)))
  (import "env" "gl_acos" (func (;4;) (type 4)))
  (import "env" "gl_atan" (func (;5;) (type 5)))
  (import "env" "gl_atan2" (func (;6;) (type 6)))
  (import "env" "gl_exp" (func (;7;) (type 7)))
  (import "env" "gl_exp2" (func (;8;) (type 8)))
  (import "env" "gl_log" (func (;9;) (type 9)))
  (import "env" "gl_log2" (func (;10;) (type 10)))
  (import "env" "gl_pow" (func (;11;) (type 11)))
  (import "env" "gl_sinh" (func (;12;) (type 12)))
  (import "env" "gl_cosh" (func (;13;) (type 13)))
  (import "env" "gl_tanh" (func (;14;) (type 14)))
  (import "env" "gl_asinh" (func (;15;) (type 15)))
  (import "env" "gl_acosh" (func (;16;) (type 16)))
  (import "env" "gl_atanh" (func (;17;) (type 17)))
  (global (;0;) (mut i32) i32.const 0)
  (global (;1;) (mut i32) i32.const 0)
  (global (;2;) (mut i32) i32.const 0)
  (global (;3;) (mut i32) i32.const 0)
  (global (;4;) (mut i32) i32.const 0)
  (global (;5;) (mut i32) i32.const 0)
  (export "main" (func 20))
  (func (;18;) (type 18) (result f32 f32 f32)
    (local i32 f32 i32)
    f32.const 0x1p+0 (;=1;)
    f32.const 0x0p+0 (;=0;)
    f32.const 0x0p+0 (;=0;)
    return
  )
  (func (;19;) (type 19)
    (local f32 f32 f32 i32 f32 i32)
    call 18
    local.set 2
    local.set 1
    local.set 0
    global.get 3
    i32.const 16
    i32.add
    local.get 0
    f32.store
    global.get 3
    i32.const 16
    i32.add
    local.get 1
    f32.store offset=4
    global.get 3
    i32.const 16
    i32.add
    local.get 2
    f32.store offset=8
    global.get 3
    global.get 3
    i32.const 16
    i32.add
    f32.load
    f32.store
    global.get 3
    global.get 3
    i32.const 16
    i32.add
    i32.const 4
    i32.add
    f32.load
    f32.store offset=4
    global.get 3
    global.get 3
    i32.const 16
    i32.add
    i32.const 8
    i32.add
    f32.load
    f32.store offset=8
    global.get 3
    f32.const 0x1p+0 (;=1;)
    f32.store offset=12
    return
  )
  (func (;20;) (type 20) (param i32 i32 i32 i32 i32 i32 i32 i32 i32)
    (local i32 f32 i32)
    local.get 3
    global.set 0
    local.get 4
    global.set 1
    local.get 5
    global.set 2
    local.get 6
    global.set 3
    local.get 7
    global.set 4
    local.get 8
    global.set 5
    call 19
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
    local.set 10
    global.get 3
    local.get 10
    f32.store offset=12
    local.set 10
    global.get 3
    local.get 10
    f32.store offset=8
    local.set 10
    global.get 3
    local.get 10
    f32.store offset=4
    local.set 10
    global.get 3
    local.get 10
    f32.store
    return
  )
)
`);
  } finally {
    gl.destroy();
  }
});
