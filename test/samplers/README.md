# Sampler & Image-Load Helper Test Plan

This is concrete, minimal-but-exhaustive test matrix and test strategies to ensure every emitted code path is covered.

## Goal 🎯  
Ensure every distinct emitted sampler/image-load code path is exercised by unit tests so regressions are caught early (format families, dims, layout, filtering, wrap, descriptor resolution, mipmaps).

---

## What to test

1. Format families (core):  
   - **RGBA8 (UNORM)** — byte loads + normalize  
   - **RGBA32F** — float loads  
   - **R32F** — single-channel float  
   - **R32UI** — unsigned integer (nearest-only handling)  
   - **RGBA32UI** — unsigned integer vector (nearest-only)  
   (expand later: RG16, RGB565, compressed formats)

2. Dimension: **2D** and **3D** samplers.

3. Layout variants: **Linear** and **Tiled8x8**.

4. Filtering/wrap/mipmap:  
   - Filters: **NEAREST**, **LINEAR** (and mipmap modes: LINEAR_MIPMAP_NEAREST, LINEAR_MIPMAP_LINEAR)  
   - Wrap: **REPEAT**, **CLAMP_TO_EDGE**  
   - Test LOD selection (generate/mark mipmaps + sample with different LODs)

5. Descriptor resolution variants:  
   - **Uniform-index model** (unit stored in uniform memory)  
   - **Direct descriptor/pointer model** (direct address)

6. Edge and validation cases: out-of-bounds coords, degenerate sizes, odd alignments (UNPACK_ALIGNMENT), integer sampling attempts with linear (must use nearest).

---

## Initial tests

- Sampler code paths to *cover all branches*:  
  - Format families (5) × Dim (2) × Layout (2) × Filter family (~2) = ~40 combinations; after removing invalid (integer+linear) → **~32 tests**.

- Image-load helper: formats (5) × Layout (2) = **~10 tests**.

- Descriptor-resolution multiplicative variants (uniform vs pointer): duplicate a subset (~20 critical combos) to validate both code paths.

- Total focused suite: **~60–70 tests** (balanced: exhaustive per-path, minimal duplicates otherwise).

---

## Test types & how to write them 🔧

1. **Shader-level functional tests (preferred)**  
   - Write small shader WASM (via existing flow) that samples a known texture and returns the sample. Instantiate, call, and assert results.
   - For float formats: assert within epsilon (e.g., 1e-6). For UNORM: tolerance ~1/255. For integer formats: exact equality.
   - Use tiled vs linear textures by setting descriptor `layout` flag and known pattern data.

2. **Unit tests for helper logic (Rust-level)**  
   - Add Rust unit tests for tile address math, byte->float conversion, and per-format decode functions (isolated functions).
   - These are fast and validate core bit-twiddling without full WASM instantiation.

3. **End-to-end render tests**  
   - Render a tiny triangle/quad using shader sampling and use `readPixels` to check output (covers pipeline + table registration + descriptor sync).

4. **Regression / fuzz tests**  
   - Parameterized tests driving random coords, wrap modes, alignments; assert invariants (e.g., integer sampler never uses linear interpolation).

5. **Performance microbench** (optional)  
   - Runs to detect huge perf regressions if sampler logic moves to Rust exports.

---

## Test implementation notes (practical) 📝

- Tests must be placed in the sampler test directory and organized so each file focuses on a single format family. Filenames should make the covered format and primary variant obvious to reviewers.
- Use the native Node test runner and the repository's test harness; follow existing test module conventions for setup and teardown so tests are isolated and deterministic.
- Use deterministic texture data and small, well-chosen patterns so expected outputs are exact where possible and predictable where approximation is required.
- Each test should assert correctness and, where applicable, branch coverage. For branch coverage of sampler/image helpers, run instrumented builds and verify that the layout/format branches were executed by the tests.
- Add a CI job to run the sampler test suite under Node (and optionally a headless browser) with coverage instrumentation enabled so regression in any helper branch is detectable in CI.

### Additional conceptual conventions (mandatory)

1. Test file scope and responsibilities
   - One test file per format family; the file can include multiple logical test cases for permutations (dimension, layout, filter), but it should be narrowly scoped to that format.

2. Assertion discipline (strict project rule)
   - Every individual test must end with exactly one assertion. When multiple verifications are logically necessary, the test must gather the results into a single structured value (object or array) and perform a single structural equality assertion against the expected structured value.

3. Float comparison policy
   - Tests comparing floating-point outputs must canonicalize values before assertion. Canonicalization should be explicit and consistent across tests (e.g., fixed decimal rounding to N places or a small epsilon-based quantization). Document the chosen precision level in the test file header.

4. Integer and UNORM formats
   - Integer samplers must be validated by exact integer equality and must be tested only in nearest sampling modes. UNORM formats may be tested as integer reads or as normalized floats but must follow canonicalization rules when floating comparisons are used.

5. Coverage targets and descriptor modes
   - Ensure tests exercise both descriptor-resolution approaches (uniform-index model and direct descriptor pointer model) for a representative subset of format/variant combinations.

6. Edge cases and property tests
   - Include tests for small textures (1×1), large textures, out-of-bounds sampling coordinates, degenerate sizes, odd unpack alignments, and validation behaviors (e.g., attempts to use linear filtering with integer samplers should be detected or validated appropriately).

7. Test checklist for PRs
   - File is named and located appropriately and targets a single format family.
   - Tests are deterministic and clean up resources on completion.
   - Each test ends with one assertion (structured equality if multiple values).
   - Floats are canonicalized and the precision policy is documented.
   - Target helper branches are exercised and reported by the coverage harness.

These conceptual rules and practices should be used by test authors to keep the sampler test suite exhaustive, consistent, and easy to review. If you want, I can now scaffold the first batch of sampler test files (following these rules) and add a fast Rust unit test for tile addressing as an immediate follow-up.

