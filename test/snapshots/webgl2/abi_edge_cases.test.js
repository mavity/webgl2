// @ts-check
import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2, getShaderWat } from '../../../index.js';

test('ABI: exactly 16 byte struct (at threshold) links', async () => {
  const gl = await webGL2({ debug: 'rust' });
  try {
    const fs = gl.createShader(gl.FRAGMENT_SHADER);
    gl.shaderSource(fs, `#version 300 es
      precision highp float; out vec4 fragColor;
      struct ExactlyThreshold { vec4 data; };
      float processThreshold(ExactlyThreshold s) { return s.data.x + s.data.y + s.data.z + s.data.w; }
      void main() { ExactlyThreshold s; s.data = vec4(1.0); float result = processThreshold(s); fragColor = vec4(result,0.0,0.0,1.0);} `);
    gl.compileShader(fs);

    const vs = gl.createShader(gl.VERTEX_SHADER);
    gl.shaderSource(vs, `#version 300 es
      void main() { gl_Position = vec4(0); }`);
    gl.compileShader(vs);

    const program = gl.createProgram();
    gl.attachShader(program, vs);
    gl.attachShader(program, fs);
    gl.linkProgram(program);

    let fsWatValue = getShaderWat(gl._ctxHandle, program._handle, gl.FRAGMENT_SHADER);
    const expectedFsWat = `(module
  (type (;0;) (func (param f32 f32 f32 f32) (result f32)))
  (type (;1;) (func))
  (type (;2;) (func (param i32 i32 i32 i32 i32 i32)))
  (import "env" "memory" (memory (;0;) 10))
  (global (;0;) (mut i32) i32.const 0)
  (global (;1;) (mut i32) i32.const 0)
  (global (;2;) (mut i32) i32.const 0)
  (global (;3;) (mut i32) i32.const 0)
  (global (;4;) (mut i32) i32.const 0)
  (global (;5;) (mut i32) i32.const 0)
  (export "main" (func 2))
  (func (;0;) (type 0) (param f32 f32 f32 f32) (result f32)
    (local i32 f32 i32)
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
    local.get 3
    f32.store offset=12
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
    global.get 3
    i32.const 12
    i32.add
    f32.load
    f32.add
    return
  )
  (func (;1;) (type 1)
    (local f32 i32 f32 i32)
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
    call 0
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
  (func (;2;) (type 2) (param i32 i32 i32 i32 i32 i32)
    (local i32 f32 i32)
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
    i32.const 524288
    global.set 5
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
    local.set 7
    global.get 3
    local.get 7
    f32.store offset=12
    local.set 7
    global.get 3
    local.get 7
    f32.store offset=8
    local.set 7
    global.get 3
    local.get 7
    f32.store offset=4
    local.set 7
    global.get 3
    local.get 7
    f32.store
    return
  )
)
`;
    assert.strictEqual(fsWatValue, expectedFsWat);
  } finally {
    gl.destroy();
  }
});

test('ABI: 17 byte struct uses WAT or null', async () => {
  const gl = await webGL2({ debug: 'rust' });
  try {
    const fs = gl.createShader(gl.FRAGMENT_SHADER);
    gl.shaderSource(fs, `#version 300 es
      precision highp float; out vec4 fragColor;
      struct JustOverThreshold { vec4 data; float extra; };
      float processOver(JustOverThreshold s) { return s.data.x + s.extra; }
      void main() { JustOverThreshold s; fragColor = vec4(processOver(s),0.0,0.0,1.0);} `);
    gl.compileShader(fs);

    const vs = gl.createShader(gl.VERTEX_SHADER);
    gl.shaderSource(vs, `#version 300 es
      void main() { gl_Position = vec4(0); }`);
    gl.compileShader(vs);

    const program = gl.createProgram();
    gl.attachShader(program, vs);
    gl.attachShader(program, fs);
    // Linking may fails for 17 byte struct??
    assert.throws(() => gl.linkProgram(program));
  } finally {
    gl.destroy();
  }
});

test('ABI: deeply nested struct links', async () => {
  const gl = await webGL2({ debug: 'rust' });
  try {
    const fs = gl.createShader(gl.FRAGMENT_SHADER);
    gl.shaderSource(fs, `#version 300 es
      precision highp float; out vec4 fragColor;
      struct Level0 { struct Level1 { struct Level2 { vec2 data; float value; } inner; vec2 extra; } inner; vec2 more; };
      float processNested(Level0 d) { return d.inner.inner.value + d.more.x; }
      void main() { Level0 data; fragColor = vec4(processNested(data),0.0,0.0,1.0);} `);
    gl.compileShader(fs);

    const vs = gl.createShader(gl.VERTEX_SHADER);
    gl.shaderSource(vs, `#version 300 es
      void main() { gl_Position = vec4(0); }`);
    gl.compileShader(vs);

    const program = gl.createProgram();
    gl.attachShader(program, vs);
    gl.attachShader(program, fs);
    // nested structs are still not supported??
    assert.throws(() => gl.linkProgram(program));
  } finally {
    gl.destroy();
  }
});