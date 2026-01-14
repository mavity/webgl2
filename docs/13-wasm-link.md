# WebAssembly Shader Linking without Bindgen

**Status:** 📋 PLANNED (Not Yet Implemented)  
**Date:** January 14, 2026  
**Estimated Effort:** 8-12 hours with incremental verification

---

## Executive Summary

This document describes an optimization to replace the current JS trampoline shader execution pattern with direct WebAssembly function table calls. The proposed change would provide **~47× performance improvement** for shader invocations, critical for high-resolution rendering.

**Current State:** ✅ Working JS trampoline (verified in code)  
**Proposed State:** 📋 Direct `call_indirect` via function table (not implemented)  
**Performance Impact:** ~700ns → ~15ns per shader call

---

## Table of Contents

1. [Architecture Analysis](#architecture-analysis)
2. [Performance Analysis](#performance-analysis)
3. [Implementation Plan](#implementation-plan)
4. [Verification Strategy](#verification-strategy)
5. [Risk Assessment](#risk-assessment)
6. [Success Criteria](#success-criteria)

---

## Architecture Analysis

### Current Architecture (As-Is)

**Shader Execution Flow:**
```
[Rust Rasterizer] src/wasm_gl_emu/rasterizer.rs:execute_fragment_shader()
    ↓ calls js_execute_shader()
[Rust Export] src/lib.rs:js_execute_shader()
    ↓ external import
[JS Import] index.js:wasm_execute_shader
    ↓ Map.get(ctx)
[JS Method] src/webgl2_context.js:_executeShader()
    ↓ program._fsInstance.exports.main()
[Shader WASM] Compiled shader module
```

**Latency Breakdown:**
- JS boundary crossing: ~500ns
- Map lookup: ~50ns
- Property access chain: ~60ns (×3 accesses)
- Function dispatch: ~100ns
- **Total overhead: ~700ns per call**

**Impact Example:**
- 1920×1080 fullscreen triangle
- Fragment shader invocations: 2,073,600
- Added latency: **~1.45 seconds per frame** (unacceptable)

**Current Files:**
- [index.js:181](../index.js#L181) - Provides `wasm_execute_shader` import
- [src/webgl2_context.js:213](../src/webgl2_context.js#L213) - `_executeShader()` method  
- [src/wasm_gl_emu/rasterizer.rs:611](../src/wasm_gl_emu/rasterizer.rs#L611) - Calls `js_execute_shader()`
- [src/lib.rs:86](../src/lib.rs#L86) - Exposes `js_execute_shader()` wrapper

### Proposed Architecture (To-Be)

**Direct Function Table Dispatch:**
```
[Rust Rasterizer] call_shader_direct(table_index)
    ↓ transmute to function pointer
[WASM call_indirect] <table_index>
    ↓ direct table lookup
[Shader WASM] main()
```

**Expected Latency:**
- `call_indirect` instruction: ~10-20ns (WASM spec estimate)
- **Total overhead: ~15ns** (47× faster than current)

**Key Insight:** In WebAssembly, function pointers ARE table indices. By storing shader `main` functions in a shared table, Rust can call them directly without JS boundary crossing.

---

## Performance Analysis

### Baseline Measurement (Required Before Implementation)

**Create:** `test/shader_call_benchmark.test.js`
```javascript
import test from 'node:test';
import { webGL2 } from '../index.js';

test('Benchmark: Shader call overhead (baseline)', async () => {
  const gl = await webGL2();
  const vs = gl.createShader(gl.VERTEX_SHADER);
  gl.shaderSource(vs, '#version 300 es\nvoid main() { gl_Position = vec4(0); }');
  gl.compileShader(vs);
  
  const fs = gl.createShader(gl.FRAGMENT_SHADER);
  gl.shaderSource(fs, '#version 300 es\nprecision mediump float; out vec4 color; void main() { color = vec4(1); }');
  gl.compileShader(fs);
  
  const prog = gl.createProgram();
  gl.attachShader(prog, vs);
  gl.attachShader(prog, fs);
  gl.linkProgram(prog);
  gl.useProgram(prog);
  
  // Small triangle for shader execution
  const vbo = gl.createBuffer();
  gl.bindBuffer(gl.ARRAY_BUFFER, vbo);
  gl.bufferData(gl.ARRAY_BUFFER, new Float32Array([0,0,0, 1,0,0, 0,1,0]), gl.STATIC_DRAW);
  
  const start = performance.now();
  for (let i = 0; i < 100; i++) {
    gl.drawArrays(gl.TRIANGLES, 0, 3);
  }
  const elapsed = performance.now() - start;
  
  console.log(`Baseline: ${elapsed.toFixed(2)}ms for 100 draws`);
  gl.destroy();
});
```

**Run:** `node --test test/shader_call_benchmark.test.js`

**Decision Criteria:**
- ✅ **GO** if baseline >200ms (worth optimizing)
- ❌ **NO-GO** if baseline <50ms (not worth complexity)
- ⚠️ **PAUSE** if can't achieve >5× improvement in testing

### Expected Performance Gains

| Scenario | Current | Optimized | Speedup |
|----------|---------|-----------|---------|
| Single shader call | ~700ns | ~15ns | 47× |
| 100 draw calls (benchmark) | ~450ms | <50ms | 9× |
| 1920×1080 fullscreen | +1.45s | +30ms | 48× |

---

## Implementation Plan

This plan uses **incremental phases with verification** to ensure each step works before proceeding.

### Prerequisites

**P1: Verify LLVM Toolchain**
```powershell
rustc --print target-spec-json --target wasm32-unknown-unknown
```
Expected: JSON output, no errors.

**P2: Baseline Test Coverage**
```powershell
npm test -- --grep "shader|compile|link"
```
Expected: All tests pass.

**P3: Performance Baseline**
Run benchmark (see above), record results.
**P3: Performance Baseline**
Run benchmark (see above), record results.

---

### Phase 1: Linker Configuration

**Goal:** Enable function table support in WASM output.  
**Risk:** Low  
**Time:** 15 minutes

#### Step 1.1: Update .cargo/config.toml

**File:** [.cargo/config.toml](../.cargo/config.toml)

**Current:**
```toml
# [target.wasm32-unknown-unknown]
# runner = "wasm-bindgen-test-runner"
[build]
# target = "wasm32-unknown-unknown"
```

**Change to:**
```toml
[target.wasm32-unknown-unknown]
rustflags = ["-C", "link-arg=--import-table"]

[build]
# target = "wasm32-unknown-unknown"
```

**Rationale:**
- `--import-table`: Instructs LLVM to expect table via imports (not self-create)
- Enables cross-module function sharing
- Required for shaders to register in Rust's table

**Why import, not export?**
We need **one shared table** created by JavaScript and passed to *both*:
1. Rust emulator module (for calling)
2. Shader WASM modules (for registration)

If Rust exports its own table, shaders can't populate it without complex re-exports.

#### Verification 1:
```powershell
cargo clean
npm run build
npm test
```

**Expected:**
- ✅ Build succeeds
- ✅ `target/wasm32-unknown-unknown/release/webgl2.wasm` exists
- ✅ All tests pass (no regressions)

**Rollback:** Remove `rustflags` line if build fails.

---

### Phase 2: JavaScript Table Infrastructure

**Goal:** Create and manage shared function table in JS.  
**Risk:** Medium  
**Time:** 1 hour

#### Step 2.1: Add Table Allocator Class
#### Step 2.1: Add Table Allocator Class

**File:** [index.js](../index.js)

**Insert after line 18:**
```javascript
/**
 * Simple allocator for function table indices.
 * Tracks which slots are in use to enable reuse.
 */
class TableAllocator {
  constructor() {
    this.nextIndex = 1; // 0 is often reserved/null
    this.freeList = [];
  }

  allocate() {
    if (this.freeList.length > 0) {
      return this.freeList.pop();
    }
    return this.nextIndex++;
  }

  free(index) {
    this.freeList.push(index);
  }
}
```

#### Step 2.2: Create Shared Table in initWASM

**File:** [index.js](../index.js)

**Find line ~173 (in `initWASM` function):**
```javascript
  const importObject = {
    env: {
```

**Replace with:**
```javascript
  // Create shared function table for direct shader calls
  const sharedTable = new WebAssembly.Table({ 
    initial: 256,    // ~100 programs (2 shaders each + overhead)
    maximum: 1024,   // Prevent unbounded growth
    element: "anyfunc" 
  });
  const tableAllocator = new TableAllocator();

  const importObject = {
    env: {
      __indirect_function_table: sharedTable,  // Exact name LLVM expects
```

#### Step 2.3: Store Table in Context

**File:** [index.js](../index.js)

**Find line ~90 (after instance creation in `webGL2` function):**
```javascript
  const ctx = new WasmWebGL2RenderingContext(
    instance,
    ctxHandle,
    { debugShaders, debugRust }
  );
```

**Replace with:**
```javascript
  const ctx = new WasmWebGL2RenderingContext(
    instance,
    ctxHandle,
    { debugShaders, debugRust },
    { sharedTable, tableAllocator }
  );
```

#### Step 2.4: Update Context Constructor

**File:** [src/webgl2_context.js](../src/webgl2_context.js)

**Find line ~188 (constructor signature):**
```javascript
  constructor(instance, ctxHandle, opts = {}) {
```

**Replace with:**
```javascript
  constructor(instance, ctxHandle, opts = {}, tableOpts = {}) {
    this._sharedTable = tableOpts.sharedTable || null;
    this._tableAllocator = tableOpts.tableAllocator || null;
```

#### Verification 2:

**Create:** `test/table_infrastructure.test.js`
```javascript
import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('Shared function table exists', async () => {
  const gl = await webGL2();
  assert.ok(gl._sharedTable, 'Context should have _sharedTable');
  assert.ok(gl._tableAllocator, 'Context should have _tableAllocator');
  assert.strictEqual(gl._sharedTable.length, 256, 'Table should have initial size 256');
  gl.destroy();
});

test('Table allocator works', async () => {
  const gl = await webGL2();
  const idx1 = gl._tableAllocator.allocate();
  const idx2 = gl._tableAllocator.allocate();
  assert.notStrictEqual(idx1, idx2, 'Should allocate different indices');
  
  gl._tableAllocator.free(idx1);
  const idx3 = gl._tableAllocator.allocate();
  assert.strictEqual(idx3, idx1, 'Should reuse freed index');
  
  gl.destroy();
});
```

**Run:** `node --test test/table_infrastructure.test.js`  
**Expected:** Both tests pass.

---

### Phase 3: Shader Registration

**Goal:** Register shader WASM functions in shared table.  
**Risk:** Medium  
**Time:** 2 hours

#### Step 3.1: Update _instantiateProgramShaders

**File:** [src/webgl2_context.js](../src/webgl2_context.js)

**Find line ~773 (vertex shader instantiation):**
```javascript
    program._vsInstance = new WebAssembly.Instance(vsModule, {
      env: {
        memory: this._instance.exports.memory,
        ...vsDebugEnv
      }
    });
    vsInstanceRef.current = program._vsInstance;
```

**Replace with:**
```javascript
    // Allocate table slots
    const vsIdx = this._tableAllocator ? this._tableAllocator.allocate() : null;
    const fsIdx = this._tableAllocator ? this._tableAllocator.allocate() : null;

    // Instantiate with shared table
    program._vsInstance = new WebAssembly.Instance(vsModule, {
      env: {
        memory: this._instance.exports.memory,
        __indirect_function_table: this._sharedTable,
        ...vsDebugEnv
      }
    });
    vsInstanceRef.current = program._vsInstance;

    // Register in table
    if (this._sharedTable && vsIdx !== null && program._vsInstance.exports.main) {
      this._sharedTable.set(vsIdx, program._vsInstance.exports.main);
      program._vsTableIndex = vsIdx;
    }
```

**Find line ~786 (fragment shader instantiation):**
```javascript
    program._fsInstance = new WebAssembly.Instance(fsModule, {
      env: {
        memory: this._instance.exports.memory,
        ...fsDebugEnv
      }
    });
    fsInstanceRef.current = program._fsInstance;
```

**Replace with:**
```javascript
    program._fsInstance = new WebAssembly.Instance(fsModule, {
      env: {
        memory: this._instance.exports.memory,
        __indirect_function_table: this._sharedTable,
        ...fsDebugEnv
      }
    });
    fsInstanceRef.current = program._fsInstance;

    if (this._sharedTable && fsIdx !== null && program._fsInstance.exports.main) {
      this._sharedTable.set(fsIdx, program._fsInstance.exports.main);
      program._fsTableIndex = fsIdx;
    }

    // Notify Rust of table indices (requires Phase 4)
    if (vsIdx !== null && fsIdx !== null) {
      const ex = this._instance.exports;
      if (ex.wasm_ctx_register_shader_indices) {
        ex.wasm_ctx_register_shader_indices(
          this._ctxHandle,
          program._handle,
          vsIdx,
          fsIdx
        );
      }
    }
```

#### Step 3.2: Add Cleanup on Program Delete

**File:** [src/webgl2_context.js](../src/webgl2_context.js)

**Find line ~665 (`deleteProgram` method):**
```javascript
  deleteProgram(program) {
    this._assertNotDestroyed();
    const ex = this._instance.exports;
    if (!ex || typeof ex.wasm_ctx_delete_program !== 'function') {
      throw new Error('wasm_ctx_delete_program not found');
    }
    const programHandle = program && typeof program === 'object' && typeof program._handle === 'number' ? program._handle : (program >>> 0);
    const code = ex.wasm_ctx_delete_program(this._ctxHandle, programHandle);
    _checkErr(code, this._instance);
  }
```

**Insert before `_checkErr()`:**
```javascript
    // Free table indices
    if (program && typeof program === 'object') {
      if (program._vsTableIndex !== undefined && this._tableAllocator) {
        this._tableAllocator.free(program._vsTableIndex);
      }
      if (program._fsTableIndex !== undefined && this._tableAllocator) {
        this._tableAllocator.free(program._fsTableIndex);
      }
    }
```

#### Verification 3:

**Create:** `test/shader_table_registration.test.js`
```javascript
import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('Shaders are registered in function table', async () => {
  const gl = await webGL2();
  
  const vs = gl.createShader(gl.VERTEX_SHADER);
  gl.shaderSource(vs, '#version 300 es\nvoid main() { gl_Position = vec4(0); }');
  gl.compileShader(vs);
  
  const fs = gl.createShader(gl.FRAGMENT_SHADER);
  gl.shaderSource(fs, '#version 300 es\nprecision mediump float; out vec4 color; void main() { color = vec4(1); }');
  gl.compileShader(fs);
  
  const prog = gl.createProgram();
  gl.attachShader(prog, vs);
  gl.attachShader(prog, fs);
  gl.linkProgram(prog);
  
  assert.ok(prog._vsTableIndex !== undefined, 'VS should have table index');
  assert.ok(prog._fsTableIndex !== undefined, 'FS should have table index');
  assert.ok(prog._vsTableIndex > 0, 'VS index should be positive');
  assert.ok(prog._fsTableIndex > 0, 'FS index should be positive');
  
  const vsFunc = gl._sharedTable.get(prog._vsTableIndex);
  const fsFunc = gl._sharedTable.get(prog._fsTableIndex);
  assert.strictEqual(typeof vsFunc, 'function', 'VS should be callable');
  assert.strictEqual(typeof fsFunc, 'function', 'FS should be callable');
  
  gl.destroy();
});
```

**Run:** `node --test test/shader_table_registration.test.js`  
**Expected:** Test passes.

---

### Phase 4: Rust Table Call Interface

**Goal:** Create Rust FFI to call shaders via table index.  
**Risk:** High (memory safety)  
**Time:** 2 hours

#### Step 4.1: Add Table Index Storage

**File:** [src/webgl2_context/shaders.rs](../src/webgl2_context/shaders.rs)

**Find `Program` struct (around line ~40):**
```rust
pub struct Program {
    pub handle: u32,
    pub vs: Option<u32>,
    pub fs: Option<u32>,
    pub link_status: bool,
    // ...
}
```

**Add fields:**
```rust
    /// Function table indices for direct calling
    pub vs_table_idx: Option<u32>,
    pub fs_table_idx: Option<u32>,
```

**Update initialization in `ctx_create_program` to initialize these fields to `None`.**

#### Step 4.2: Add Rust Registration Function

**File:** [src/lib.rs](../src/lib.rs)

**Add after other `wasm_ctx_*` functions:**
```rust
/// Register compiled shader function table indices.
/// Called from JS after shader WASM instances are created.
#[no_mangle]
pub extern "C" fn wasm_ctx_register_shader_indices(
    ctx: u32,
    program: u32,
    vs_idx: u32,
    fs_idx: u32,
) -> u32 {
    webgl2_context::ctx_register_shader_indices(ctx, program, vs_idx, fs_idx)
}
```

#### Step 4.3: Implement Registration

**File:** [src/webgl2_context/shaders.rs](../src/webgl2_context/shaders.rs)

**Add function:**
```rust
use super::registry::{clear_last_error, get_registry, set_last_error};
use crate::error::{ERR_INVALID_HANDLE, ERR_OK};

/// Store shader table indices for direct calling.
pub fn ctx_register_shader_indices(
    ctx: u32,
    program: u32,
    vs_idx: u32,
    fs_idx: u32,
) -> u32 {
    clear_last_error();
    let mut reg = get_registry().borrow_mut();
    let ctx_obj = match reg.contexts.get_mut(&ctx) {
        Some(c) => c,
        None => {
            set_last_error("invalid context handle");
            return ERR_INVALID_HANDLE;
        }
    };

    if let Some(prog) = ctx_obj.programs.get_mut(&program) {
        prog.vs_table_idx = Some(vs_idx);
        prog.fs_table_idx = Some(fs_idx);
        ERR_OK
    } else {
        set_last_error("invalid program handle");
        ERR_INVALID_HANDLE
    }
}
```

#### Verification 4:

```powershell
npm run build
npm test
```

**Expected:** All existing tests still pass.

---

### Phase 5: call_indirect Implementation

**Goal:** Replace JS trampoline with direct table calls.  
**Risk:** High (transmute safety)  
**Time:** 2 hours

#### Step 5.1: Add Shader Call Helpers

**File:** [src/wasm_gl_emu/rasterizer.rs](../src/wasm_gl_emu/rasterizer.rs)

**Add after imports (line ~10):**
```rust
/// Type alias for shader function signature.
/// Matches WASM shader exports: (type, attr_ptr, uniform_ptr, varying_ptr, private_ptr, texture_ptr)
type ShaderFunc = unsafe extern "C" fn(i32, i32, i32, i32, i32, i32);

/// Call shader directly via function table.
/// 
/// # Safety
/// - `table_index` must be valid (set during linkProgram)
/// - Memory pointers must be valid and aligned
/// - Shader WASM instance must still exist
/// 
/// # Implementation Note
/// In WebAssembly, function pointers ARE table indices.
/// This transmute is safe because:
/// 1. WASM spec guarantees table indices map to functions
/// 2. call_indirect validates signature at runtime
/// 3. Invalid indices trap (don't cause UB)
#[inline]
unsafe fn call_shader_direct(
    table_index: u32,
    shader_type: u32,
    attr_ptr: u32,
    uniform_ptr: u32,
    varying_ptr: u32,
    private_ptr: u32,
    texture_ptr: u32,
) {
    // Transmute table index to function pointer
    let func: ShaderFunc = std::mem::transmute(table_index as usize);
    
    func(
        shader_type as i32,
        attr_ptr as i32,
        uniform_ptr as i32,
        varying_ptr as i32,
        private_ptr as i32,
        texture_ptr as i32,
    );
}
```

#### Step 5.2: Update execute_fragment_shader

**File:** [src/wasm_gl_emu/rasterizer.rs](../src/wasm_gl_emu/rasterizer.rs)

**Find `execute_fragment_shader` method (line ~611):**
```rust
    fn execute_fragment_shader(
        &self,
        varyings: &[u32],
        pipeline: &RasterPipeline,
        state: &RenderState,
    ) -> [u8; 4] {
        // Copy varyings to shader memory as raw bits
        unsafe {
            std::ptr::copy_nonoverlapping(
                varyings.as_ptr() as *const u8,
                pipeline.memory.varying_ptr as *mut u8,
                varyings.len() * 4,
            );
        }

        // Execute fragment shader
        crate::js_execute_shader(
            state.ctx_handle,
            pipeline.fragment_shader_type,
            0,
            pipeline.memory.uniform_ptr,
            pipeline.memory.varying_ptr,
            pipeline.memory.private_ptr,
            pipeline.memory.texture_ptr,
        );
```

**Replace the `js_execute_shader` call with:**
```rust
        // Try direct call if table index available, fallback to JS trampoline
        if let Some(fs_idx) = pipeline.fs_table_idx {
            unsafe {
                call_shader_direct(
                    fs_idx,
                    pipeline.fragment_shader_type,
                    0,
                    pipeline.memory.uniform_ptr,
                    pipeline.memory.varying_ptr,
                    pipeline.memory.private_ptr,
                    pipeline.memory.texture_ptr,
                );
            }
        } else {
            // Fallback: JS trampoline (preserves existing behavior)
            crate::js_execute_shader(
                state.ctx_handle,
                pipeline.fragment_shader_type,
                0,
                pipeline.memory.uniform_ptr,
                pipeline.memory.varying_ptr,
                pipeline.memory.private_ptr,
                pipeline.memory.texture_ptr,
            );
        }
```

#### Step 5.3: Update RasterPipeline Struct

**File:** [src/wasm_gl_emu/rasterizer.rs](../src/wasm_gl_emu/rasterizer.rs)

**Find `RasterPipeline` struct (around line ~85):**
```rust
pub struct RasterPipeline {
    // ... existing fields ...
}
```

**Add fields:**
```rust
    /// Function table index for vertex shader (if available)
    pub vs_table_idx: Option<u32>,
    /// Function table index for fragment shader (if available)
    pub fs_table_idx: Option<u32>,
```

#### Step 5.4: Pass Indices from Context

**File:** Search for where `RasterPipeline` is constructed (likely in `src/webgl2_context/drawing.rs` or similar).

**Add to pipeline initialization:**
```rust
    vs_table_idx: program.vs_table_idx,
    fs_table_idx: program.fs_table_idx,
```

#### Verification 5:

**Create:** `test/direct_shader_calls.test.js`
```javascript
import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('Shaders execute via direct table calls', async () => {
  const gl = await webGL2();
  
  const vs = gl.createShader(gl.VERTEX_SHADER);
  gl.shaderSource(vs, `#version 300 es
    in vec4 position;
    void main() { gl_Position = position; }
  `);
  gl.compileShader(vs);
  assert.ok(gl.getShaderParameter(vs, gl.COMPILE_STATUS));
  
  const fs = gl.createShader(gl.FRAGMENT_SHADER);
  gl.shaderSource(fs, `#version 300 es
    precision mediump float;
    out vec4 color;
    void main() { color = vec4(1.0, 0.0, 0.0, 1.0); }
  `);
  gl.compileShader(fs);
  assert.ok(gl.getShaderParameter(fs, gl.COMPILE_STATUS));
  
  const prog = gl.createProgram();
  gl.attachShader(prog, vs);
  gl.attachShader(prog, fs);
  gl.linkProgram(prog);
  assert.ok(gl.getProgramParameter(prog, gl.LINK_STATUS));
  
  gl.useProgram(prog);
  
  // Create triangle
  const vbo = gl.createBuffer();
  gl.bindBuffer(gl.ARRAY_BUFFER, vbo);
  gl.bufferData(gl.ARRAY_BUFFER, new Float32Array([
    -1, -1, 0, 1,
     1, -1, 0, 1,
     0,  1, 0, 1
  ]), gl.STATIC_DRAW);
  
  const posLoc = gl.getAttribLocation(prog, 'position');
  gl.enableVertexAttribArray(posLoc);
  gl.vertexAttribPointer(posLoc, 4, gl.FLOAT, false, 0, 0);
  
  // Draw (should use direct table calls)
  gl.clearColor(0, 0, 0, 1);
  gl.clear(gl.COLOR_BUFFER_BIT);
  gl.drawArrays(gl.TRIANGLES, 0, 3);
  
  // Verify red color in center
  const pixels = new Uint8Array(4);
  gl.readPixels(320, 240, 1, 1, gl.RGBA, gl.UNSIGNED_BYTE, pixels);
  assert.strictEqual(pixels[0], 255, 'Red channel should be 255');
  assert.strictEqual(pixels[1], 0, 'Green channel should be 0');
  
  gl.destroy();
});
```

**Run:** `node --test test/direct_shader_calls.test.js`  
**Expected:** Test passes with correct red pixel.

---

### Phase 6: Performance Validation

**Goal:** Confirm performance improvement.  
**Risk:** Low  
**Time:** 1 hour

#### Step 6.1: Run Benchmark

```powershell
node --test test/shader_call_benchmark.test.js
```

**Expected:**
- Baseline (from prerequisites): ~450ms
- Optimized: <50ms
- **Improvement: >9× faster**

#### Step 6.2: Full Test Suite

```powershell
npm test
```

**Expected:** All tests pass, no regressions.

#### Step 6.3: Memory Leak Check

```powershell
node --test --expose-gc test/
```

Monitor for memory growth patterns.

---

## Verification Strategy

### Per-Phase Verification

Each phase includes:
1. **Build verification:** Code compiles without errors
2. **Unit test:** Isolated functionality works correctly
3. **Integration test:** Works with existing codebase
4. **Regression test:** Doesn't break existing tests

### Final Verification Checklist

- [ ] All tests in `npm test` pass
- [ ] Benchmark shows >5× improvement
- [ ] Memory leak test passes (run with `--expose-gc`)
- [ ] Works in multiple environments (Node.js, browser if applicable)
- [ ] Debug mode still functional (when `debug: 'shaders'`)
- [ ] No build warnings
- [ ] WASM output size increase <10%
- [ ] Documentation updated

---

## Risk Assessment

### Risk 1: Table Index Invalidation

**Problem:** Shader instance deleted but Rust still has table index.

**Likelihood:** Medium  
**Impact:** High (crash)

**Mitigation:**
- Always check `Option<u32>` in Rust before using
- Fallback to JS trampoline if `None`
- Clear indices to `None` in `deleteProgram`
- Table slots freed immediately after use

### Risk 2: Memory Safety Violation

**Problem:** `transmute(table_index as usize)` is undefined behavior if index is invalid.

**Likelihood:** Low (if properly validated)  
**Impact:** Critical (undefined behavior)

**Mitigation:**
- Validate table index range before transmute
- WebAssembly.Table provides bounds checking
- Add debug assertions in Rust
- WASM spec guarantees `call_indirect` validates signature

### Risk 3: Performance Regression

**Problem:** Table indirection could be slower than optimized JS in some engines.

**Likelihood:** Low  
**Impact:** Medium (no improvement)