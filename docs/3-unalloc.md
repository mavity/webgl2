# Ephemeral pointer WASM returns

This document records the plan for eliminating most JS `wasm_alloc`/`wasm_free` allocations for WASM→JS variable outputs by adopting a single-pointer, 16‑byte lead header convention and three per-context/thread arenas.

- WASM functions that produce variable-sized outputs will return a single `u32 ptr` pointing at payload start.
- The 16 bytes immediately preceding `ptr` are the header (little-endian):
  - bytes [ptr-16 .. ptr-13]: `len: u32` (payload length in bytes)
  - bytes [ptr-12 .. ptr-9]: `reserved: u32` (must be zero for now)
  - bytes [ptr-8  .. ptr-1]: `reserved: u64` (must be zero for now)
- `ptr == 0` indicates failure (WASM sets last-error). The pointer is **ephemeral** and valid only until the next call into WASM on the same context/thread.

Pointers returned from the functions migrated onto this plan will be **ephemeral**, **must** be copied as soon as JavaScript receives them, and **must not** be accessed after any subsequent WASM call.

That requirement is **HARD AND FIXED**, but it eliminates all risks in regards of memory growth, reentrancy and so on.

## Threading

Note that webgl2 library is **EXPLICITLY SINGLE-THREADED** and its behaviour is undefined in multithread environment. All guarantees are off if it's ever invoked that way. Also note that we use thread-local variables to denote static, as that provides best developer UX and efficiency for Rust compilation. In single-threaded environment thread-local is the same as static anyway.

---

## Arena types
Design three allocator arenas to cover the common use cases:

1. **Small fixed-size arena** (<= 128 bytes)
   - Fast inline area for short structured outputs (e.g., viewport, color clear, small parameter structs).
   - Allocator can be a fixed `Vec<u8>` or small stack buffer per-context.

2. **Coarse binary arena** (growable, per-context)
   - For larger binary blobs with chunked growth (e.g., `readPixels`, texture dumps).
   - Use a per-context `Vec<u8>` scratch buffer with growth policy (double or exact) and an optional soft cap.

3. **String/text arena**
   - For variable-length text (WAT, info logs, decompiled GLSL). Adopt the existing `WAT_STRING_STORAGE` approach strategically for all such calls.

All arenas must be allocated at 16-byte aligned linear offsets, then reserve the 16‑byte leading header immediately before the returned payload and return `payload_ptr`.

---

## API conventions
- Naming: functions that return a payload pointer should not have and `_ref` or `_ptr` suffixes indicating return convention, because this pattern is henceforth the default baseline.
- Return semantics:
  - `u32` pointer; `0` => failure (read last-error), otherwise valid payload pointer.
  - JS must copy payload out of linear memory immediately and must not assume payload lifetime beyond the next call into WASM.
- Header layout (little-endian): `len:u32` at `ptr-16`; reserved 12 bytes afterwards must be zero.
- Each API will set the len:u32 at ptr-16 whether the size of the data is statically known or not.

---

## JS helpers

There will be **NO** centralised helpers on the JS side to deal with the pattern. That also explicitly means **NO** helper functions to be introduced per-case either.

---

## Rust-side pattern
- If arena is not initialised or too small, extend it; some arenas MAY shrink if new size is much smaller (useful for variable string cases)
- Consider existing previous data in the arena gone
- `payload_ptr = arena_ptr + 16`
- write `len` at `arena_ptr` (u32 little-endian), zero remaining 12 bytes
- fill payload at `payload_ptr`
- return `payload_ptr as u32` (or `0` on error)

Ensure header fields are fully written before returning the pointer.

---

## Which `wasm_alloc`/`wasm_free` remain needed
- Keep `wasm_alloc`/`wasm_free` exported and used for **JS→WASM uploads** (textures, names, buffers) and any critical case where WASM must own a buffer beyond the call. Make a note which ones are retain alloc.
- Remove JS-side alloc/free for WASM→JS outputs migrated to the header convention.

---

## Migration plan (prioritized)

Migrate WASM host functions in groups by use case similarity. Complete one set, add unit tests before moving to another.

Each step must include unit tests and Rust functions MUST have substantial doc comments regarding the pattern.

---

## Tests

Unit tests should be written in JavaScript built-in node runner, in the same fashion we already have.
- Header correctness (len and reserved bytes zero).
- Zero-length payload supported (ptr != 0, len == 0).
- Error path: `ptr == 0` and `wasm_last_error` populated.
- Lifetime: pointer invalidated after subsequent WASM call (tests verify data is overwritten).

---

## Notes & future extensions
- The reserved 12 bytes are intentionally zero today but allow future expansion (64-bit length, epoch, arena-id or flags) without changing ABI.
- Consider debug-only epoch tags in `meta` to detect stale pointer misuse in tests.
- Document the ephemeral-pointer rule prominently in `docs/` and in function-level comments.

---


# Implementation Plan

- Add **three per-context arenas** (small, binary/blob, strings) and thread-safe helpers that produce ephemeral payload pointers with a 16‑byte leading header (len:u32 + 12 bytes reserved zero).
- Convert WASM→JS *readback* APIs in groups: **Strings/Text**, **Small fixed structs (<=128 B)**, **Binary blobs (readPixels, wasm bytes)**.
- Keep `wasm_alloc`/`wasm_free` for JS→WASM uploads and any retained ownership cases.
- Drive migration in small phases; add unit tests for header correctness, zero-length payloads, error paths and lifetime invalidation.

---

## Low-level implementation (Rust) 🔧
1. **Context fields** (modify `Context` in types.rs):
   - `small_arena: Vec<u8>` (reserve 128B, fixed).
   - `blob_arena: Vec<u8>` (growable scratch buffer).
   - `string_arena: String` or reuse existing pattern (e.g., `WAT_STRING_STORAGE`/`DECOMPILED_GLSL`) but **per-context**.
2. **Helpers** (new module, e.g., `src/webgl2_context/ephemeral.rs`):
   - `fn alloc_small(ctx: &mut Context, size: usize) -> Result<u32, Errno>`
   - `fn alloc_blob(ctx: &mut Context, size: usize) -> Result<u32, Errno>`
   - `fn alloc_string(ctx: &mut Context, s: &str) -> u32`
   - Each helper must:
     - Ensure 16‑byte alignment for returned `payload_ptr`.
     - Write header at `payload_ptr - 16`: `len:u32` (LE), then 12 zero bytes.
     - Return `payload_ptr as u32` or `0` on error and set last error via existing `set_last_error`.
3. **Arena growth policy**:
   - `small_arena`: fixed or minimal grow shrink policy.
   - `blob_arena`: grow (double or exact), optional soft cap. Overwrite previous content on re-alloc.
   - `string_arena`: reuse `String` semantics (set/replace as needed) and return pointer into its buffer.
4. **Safety note (ephemeral rule)**:
   - Document and enforce: returned pointer valid only until the next call into WASM on same context (tests will ensure overwriting happens).
   - No central JS helpers (per doc); JS will copy data immediately.

---

## API groups to convert (concrete list) 📋

Group A — Strings / Text (low risk)
- `ctx_get_shader_info_log` / exported `wasm_ctx_get_shader_info_log(ctx, shader)`  
- `ctx_get_program_info_log` / `wasm_ctx_get_program_info_log(...)`  
- `getSupportedExtensions`, `getExtension(name)`  
- `wasm_get_decompiled_glsl_*` already uses `DECOMPILED_GLSL` — adopt same per-context pattern or wrap into `string_arena`.

Group B — Small fixed-sized outputs (≤128 bytes)
- `ctx_get_parameter_v` (VIEWPORT, COLOR_CLEAR_VALUE → 4 ints / 4 floats)
- `getActiveAttrib`, `getActiveUniform` (their small descriptor structs; name remains in string arena)
- `ctx_get_program_wasm_len` (maybe replaced with a pointer-return variant instead of separate len call)

Group C — Binary / Large blobs
- `ctx_read_pixels` → return pointer to pixel data blob in `blob_arena`
- `getProgramWasm(program, shaderType)` → return pointer to binary WASM bytes
- Any other blob-like readbacks (texture dumps, responses returning large arrays)

Group D — Retain `wasm_alloc/wasm_free`
- Data uploads from JS → WASM (texture uploads, shader *source* uploads, `webgpu_context` uploads)
- Any API where WASM must retain ownership / lifetime beyond the call

---

## Migration phases & order (recommended) ⏳

Phase 1 — Prep & infra, Strings/Text
- Add arenas and low-level helpers, update `Context::new` instantiation.
- Add unit tests for helper primitives (header format, alignment, zeroing reserved bytes).
- Add doc comments for ephemeral rule.
- Migrate `getShaderInfoLog`, `getProgramInfoLog`, `getSupportedExtensions`, `getExtension`, and expose wasm exports returning `u32 ptr`.
- Update JS glue (webgl2_context.js and index.js) to call new functions, check `ptr==0`, read header at `ptr-16`, copy payload immediately.
- Tests: existing tests that call these APIs should pass after migration.

Phase 2 — Small structs
- Migrate `getParameter` and `getActive*` metadata (return pointer to small payload).
- Update tests that assert arrays/structs.

Phase 3 — Binaries & readPixels
- Migrate `readPixels` and `getProgramWasm` (largest work).
- Pay attention to packing/stride/pack-alignment rules (see 1.1.2-texture.md) when writing readPixels into the blob arena.
- Add large-data tests (e.g., full-size readPixels scenarios).

---

## JS changes (concise) 🔁
- For each migrated API, replace:
  - Old: allocate `ptr = ex.wasm_alloc(len); call wasm_func(ptr, len); memcopy; ex.wasm_free(ptr)`
  - New: call `ptr = ex.wasm_ctx_whatever(...); if(ptr==0) -> read wasm_last_error and throw; else read len = mem32[ptr-16]; read payload at `ptr` and copy immediately`.
- Ensure JS creates fresh views after any call that may grow memory (unchanged rule).
- No centralized helper — implement small per-call copy logic consistent with current style.

---

## Tests ✅ (required per migration)
- Header correctness: len set correctly and reserved 12 bytes zeroed.
- Zero-length payload: returns `ptr != 0` and `len == 0`.
- Error path: `ptr == 0` and `wasm_last_error` contains message.
- Lifetime: payload must be invalidated by a subsequent WASM call (create test that reads pointer, verifies content, then call another ephemeral-producing function to overwrite, and assert the old pointer is changed).
- Keep tests small and granular — follow existing test style and node built-in runner convention.

---

## Files to touch (concrete pointers) 📂
- Add: `src/webgl2_context/ephemeral.rs` (new helpers)
- Modify: types.rs (Context fields + init)
- Modify: registry.rs (if any global helpers needed)
- Modify/replace exports in `src/*/*.rs`:
  - shaders.rs, `program.rs`, drawing.rs, state.rs where readbacks occur
  - lib.rs to expose new exported functions
- Modify JS glue:
  - webgl2_context.js, index.js to handle pointer-return semantics
- Tests: update/extend files under test (e.g., getShaderInfoLog.test.js, `getProgramWasm.test.js`, `readPixels.test.js`)

---

## Risks & mitigations ⚠️
- Pointer invalidation must be documented and strictly tested. Add a test harness that validates misuse is detectable (test-only epoch or overwrite assertions).
- Arena reallocation may move buffer memory; ephemeral rule avoids dangling lifetime issues.
- Avoid changing APIs in large batch: do small PRs, one group per PR, with tests and doc updates.

---
