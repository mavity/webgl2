import test from 'node:test';
import assert from 'node:assert';
import { webGL2 } from '../../../index.js';

/*
 * FAIL: This test fails due to multiple missing API implementations in `src/webgl2_context.js` and the Rust backend.
 * 
 * Missing Functions:
 * 1. `gl.getVertexAttrib(index, pname)`: Not implemented in JS or Rust.
 * 2. `gl.vertexAttrib[1234]fv`: Vector variants of vertexAttrib* are not implemented in JS.
 * 3. `gl.vertexAttribI*`: Integer variants (vertexAttribI4i, vertexAttribI4ui, etc.) are not implemented in JS or Rust.
 * 
 * Root Cause:
 * The WebGL2 implementation is incomplete. The `getVertexAttrib` query is essential for this test, which verifies
 * that attribute values can be set and retrieved correctly.
 * 
 * Proposed Fix:
 * 1. Implement `wasm_ctx_get_vertex_attrib` in `src/lib.rs` and `src/webgl2_context/vaos.rs`.
 * 2. Expose `getVertexAttrib` in `src/webgl2_context.js`.
 * 3. Implement `vertexAttrib[1234]fv` in `src/webgl2_context.js` (can be wrappers around `vertexAttrib[1234]f`).
 * 4. Implement `wasm_ctx_vertex_attrib_i*` in `src/lib.rs` and `src/webgl2_context/vaos.rs`.
 * 5. Expose `vertexAttribI*` in `src/webgl2_context.js`.
 */

// Helper to generate range [0, ..., n-1]
function range(n) {
    return Array.from({ length: n }, (_, i) => i);
}

// Helper for cross product of arrays
function crossCombine(arr1, arr2) {
    const result = [];
    for (const i of arr1) {
        for (const j of arr2) {
            result.push({ ...i, ...j });
        }
    }
    return result;
}

// Simple deterministic RNG
class DRNG {
    constructor(seed = 12345) {
        this.seed = seed;
    }
    next() {
        this.seed = (this.seed * 1664525 + 1013904223) % 4294967296;
        return this.seed / 4294967296;
    }
    next_unorm() {
        return this.next();
    }
    next_u32() {
        return Math.floor(this.next() * 4294967296) >>> 0;
    }
}

test('WebGL vertexAttrib Conformance Tests', { skip: true }, async (t) => {
    const gl = await webGL2();
    const drng = new DRNG();

    const numVertexAttribs = gl.getParameter(gl.MAX_VERTEX_ATTRIBS);
    assert.ok(numVertexAttribs >= 16, 'MAX_VERTEX_ATTRIBS should be at least 16');

    // Helper to check GL error
    function checkError(expected, msg) {
        const err = gl.getError();
        assert.strictEqual(err, expected, msg || `Error should be ${expected}, got ${err}`);
    }

    // Helper to check across all indices
    const checkAcrossAllIndices = (setupFn, expectedValues, expectedType) => {
        const results = [];
        for (let ii = 0; ii < numVertexAttribs; ++ii) {
            setupFn(ii);
            const attrib = gl.getVertexAttrib(ii, gl.CURRENT_VERTEX_ATTRIB);
            results.push({
                index: ii,
                type: attrib?.constructor?.name,
                values: Array.from(attrib || [])
            });
        }
        // Generate expected array for comparison
        const expected = [];
        for (let ii = 0; ii < numVertexAttribs; ++ii) {
            expected.push({
                index: ii,
                type: expectedType.name,
                values: expectedValues
            });
        }
        return { actual: results, expected };
    };

    await t.test('vertexAttrib1fv', () => {
        const res1 = checkAcrossAllIndices((i) => gl.vertexAttrib1fv(i, [1]), [1, 0, 0, 1], Float32Array);
        const res2 = checkAcrossAllIndices((i) => gl.vertexAttrib1fv(i, new Float32Array([-1])), [-1, 0, 0, 1], Float32Array);

        assert.deepStrictEqual(
            { case1: res1.actual, case2: res2.actual },
            { case1: res1.expected, case2: res2.expected }
        );
        checkError(gl.NO_ERROR);
    });

    await t.test('vertexAttrib2fv', () => {
        const res1 = checkAcrossAllIndices((i) => gl.vertexAttrib2fv(i, [1, 2]), [1, 2, 0, 1], Float32Array);
        const res2 = checkAcrossAllIndices((i) => gl.vertexAttrib2fv(i, new Float32Array([1, -2])), [1, -2, 0, 1], Float32Array);

        assert.deepStrictEqual(
            { case1: res1.actual, case2: res2.actual },
            { case1: res1.expected, case2: res2.expected }
        );
        checkError(gl.NO_ERROR);
    });

    await t.test('vertexAttrib3fv', () => {
        const res1 = checkAcrossAllIndices((i) => gl.vertexAttrib3fv(i, [1, 2, 3]), [1, 2, 3, 1], Float32Array);
        const res2 = checkAcrossAllIndices((i) => gl.vertexAttrib3fv(i, new Float32Array([1, -2, 3])), [1, -2, 3, 1], Float32Array);

        assert.deepStrictEqual(
            { case1: res1.actual, case2: res2.actual },
            { case1: res1.expected, case2: res2.expected }
        );
        checkError(gl.NO_ERROR);
    });

    await t.test('vertexAttrib4fv', () => {
        const res1 = checkAcrossAllIndices((i) => gl.vertexAttrib4fv(i, [1, 2, 3, 4]), [1, 2, 3, 4], Float32Array);
        const res2 = checkAcrossAllIndices((i) => gl.vertexAttrib4fv(i, new Float32Array([1, 2, -3, 4])), [1, 2, -3, 4], Float32Array);

        assert.deepStrictEqual(
            { case1: res1.actual, case2: res2.actual },
            { case1: res1.expected, case2: res2.expected }
        );
        checkError(gl.NO_ERROR);
    });

    await t.test('vertexAttrib1f', () => {
        const res = checkAcrossAllIndices((i) => gl.vertexAttrib1f(i, 5), [5, 0, 0, 1], Float32Array);
        assert.deepStrictEqual(res.actual, res.expected);
        checkError(gl.NO_ERROR);
    });

    await t.test('vertexAttrib2f', () => {
        const res = checkAcrossAllIndices((i) => gl.vertexAttrib2f(i, 6, 7), [6, 7, 0, 1], Float32Array);
        assert.deepStrictEqual(res.actual, res.expected);
        checkError(gl.NO_ERROR);
    });

    await t.test('vertexAttrib3f', () => {
        const res = checkAcrossAllIndices((i) => gl.vertexAttrib3f(i, 7, 8, 9), [7, 8, 9, 1], Float32Array);
        assert.deepStrictEqual(res.actual, res.expected);
        checkError(gl.NO_ERROR);
    });

    await t.test('vertexAttrib4f', () => {
        const res = checkAcrossAllIndices((i) => gl.vertexAttrib4f(i, 6, 7, 8, 9), [6, 7, 8, 9], Float32Array);
        assert.deepStrictEqual(res.actual, res.expected);
        checkError(gl.NO_ERROR);
    });

    await t.test('vertexAttribI4i', () => {
        const res = checkAcrossAllIndices((i) => gl.vertexAttribI4i(i, -1, 0, 1, 2), [-1, 0, 1, 2], Int32Array);
        assert.deepStrictEqual(res.actual, res.expected);
        checkError(gl.NO_ERROR);
    });

    await t.test('vertexAttribI4ui', () => {
        const res = checkAcrossAllIndices((i) => gl.vertexAttribI4ui(i, 0, 1, 2, 3), [0, 1, 2, 3], Uint32Array);
        assert.deepStrictEqual(res.actual, res.expected);
        checkError(gl.NO_ERROR);
    });

    await t.test('vertexAttribI4iv', () => {
        const res1 = checkAcrossAllIndices((i) => gl.vertexAttribI4iv(i, [-1, 0, 1, 2]), [-1, 0, 1, 2], Int32Array);
        const res2 = checkAcrossAllIndices((i) => gl.vertexAttribI4iv(i, new Int32Array([1, 0, -1, 2])), [1, 0, -1, 2], Int32Array);

        assert.deepStrictEqual(
            { case1: res1.actual, case2: res2.actual },
            { case1: res1.expected, case2: res2.expected }
        );
        checkError(gl.NO_ERROR);
    });

    await t.test('vertexAttribI4uiv', () => {
        const res1 = checkAcrossAllIndices((i) => gl.vertexAttribI4uiv(i, [0, 1, 2, 3]), [0, 1, 2, 3], Uint32Array);
        const res2 = checkAcrossAllIndices((i) => gl.vertexAttribI4uiv(i, new Uint32Array([0, 2, 1, 3])), [0, 2, 1, 3], Uint32Array);

        assert.deepStrictEqual(
            { case1: res1.actual, case2: res2.actual },
            { case1: res1.expected, case2: res2.expected }
        );
        checkError(gl.NO_ERROR);
    });

    await t.test('Checking out-of-range vertexAttrib index', () => {
        gl.getVertexAttrib(numVertexAttribs, gl.CURRENT_VERTEX_ATTRIB);
        checkError(gl.INVALID_VALUE);

        gl.vertexAttrib1fv(numVertexAttribs, [1]);
        checkError(gl.INVALID_VALUE);

        gl.vertexAttrib1fv(numVertexAttribs, new Float32Array([-1]));
        checkError(gl.INVALID_VALUE);

        gl.vertexAttrib2fv(numVertexAttribs, [1, 2]);
        checkError(gl.INVALID_VALUE);

        gl.vertexAttrib2fv(numVertexAttribs, new Float32Array([1, -2]));
        checkError(gl.INVALID_VALUE);

        gl.vertexAttrib3fv(numVertexAttribs, [1, 2, 3]);
        checkError(gl.INVALID_VALUE);

        gl.vertexAttrib3fv(numVertexAttribs, new Float32Array([1, -2, 3]));
        checkError(gl.INVALID_VALUE);

        gl.vertexAttrib4fv(numVertexAttribs, [1, 2, 3, 4]);
        checkError(gl.INVALID_VALUE);

        gl.vertexAttrib4fv(numVertexAttribs, new Float32Array([1, 2, -3, 4]));
        checkError(gl.INVALID_VALUE);

        gl.vertexAttrib1f(numVertexAttribs, 5);
        checkError(gl.INVALID_VALUE);

        gl.vertexAttrib2f(numVertexAttribs, 6, 7);
        checkError(gl.INVALID_VALUE);

        gl.vertexAttrib3f(numVertexAttribs, 7, 8, 9);
        checkError(gl.INVALID_VALUE);

        gl.vertexAttrib4f(numVertexAttribs, 6, 7, 8, 9);
        checkError(gl.INVALID_VALUE);

        gl.vertexAttribI4i(numVertexAttribs, -1, 0, 1, 2);
        checkError(gl.INVALID_VALUE);

        gl.vertexAttribI4ui(numVertexAttribs, 0, 1, 2, 3);
        checkError(gl.INVALID_VALUE);

        gl.vertexAttribI4iv(numVertexAttribs, [-1, 0, 1, 2]);
        checkError(gl.INVALID_VALUE);

        gl.vertexAttribI4iv(numVertexAttribs, new Int32Array([1, 0, -1, 2]));
        checkError(gl.INVALID_VALUE);

        gl.vertexAttribI4uiv(numVertexAttribs, [0, 1, 2, 3]);
        checkError(gl.INVALID_VALUE);

        gl.vertexAttribI4uiv(numVertexAttribs, new Uint32Array([0, 2, 1, 3]));
        checkError(gl.INVALID_VALUE);
    });

    await t.test('Checking invalid array lengths', () => {
        const idx = numVertexAttribs - 1;

        gl.vertexAttrib1fv(idx, []);
        checkError(gl.INVALID_VALUE);

        gl.vertexAttrib1fv(idx, new Float32Array([]));
        checkError(gl.INVALID_VALUE);

        gl.vertexAttrib2fv(idx, [1]);
        checkError(gl.INVALID_VALUE);

        gl.vertexAttrib2fv(idx, new Float32Array([1]));
        checkError(gl.INVALID_VALUE);

        gl.vertexAttrib3fv(idx, [1, 2]);
        checkError(gl.INVALID_VALUE);

        gl.vertexAttrib3fv(idx, new Float32Array([1, -2]));
        checkError(gl.INVALID_VALUE);

        gl.vertexAttrib4fv(idx, [1, 2, 3]);
        checkError(gl.INVALID_VALUE);

        gl.vertexAttrib4fv(idx, new Float32Array([1, 2, -3]));
        checkError(gl.INVALID_VALUE);

        gl.vertexAttribI4iv(idx, [-1, 0, 1]);
        checkError(gl.INVALID_VALUE);

        gl.vertexAttribI4iv(idx, new Int32Array([1, 0, -1]));
        checkError(gl.INVALID_VALUE);

        gl.vertexAttribI4uiv(idx, [0, 1, 2]);
        checkError(gl.INVALID_VALUE);

        gl.vertexAttribI4uiv(idx, new Uint32Array([0, 2, 1]));
        checkError(gl.INVALID_VALUE);
    });

    await t.test('Checking round-tripping of valid random values', () => {
        const FUNCS = [
            { func_name: 'vertexAttrib1f', val_count: 1, array_ctor: Float32Array },
            { func_name: 'vertexAttrib2f', val_count: 2, array_ctor: Float32Array },
            { func_name: 'vertexAttrib3f', val_count: 3, array_ctor: Float32Array },
            { func_name: 'vertexAttrib4f', val_count: 4, array_ctor: Float32Array },
            { func_name: 'vertexAttrib1fv', val_count: 1, array_ctor: Float32Array },
            { func_name: 'vertexAttrib2fv', val_count: 2, array_ctor: Float32Array },
            { func_name: 'vertexAttrib3fv', val_count: 3, array_ctor: Float32Array },
            { func_name: 'vertexAttrib4fv', val_count: 4, array_ctor: Float32Array },
            { func_name: 'vertexAttribI4i', val_count: 4, array_ctor: Int32Array },
            { func_name: 'vertexAttribI4iv', val_count: 4, array_ctor: Int32Array },
            { func_name: 'vertexAttribI4ui', val_count: 4, array_ctor: Uint32Array },
            { func_name: 'vertexAttribI4uiv', val_count: 4, array_ctor: Uint32Array },
        ];

        const TESTS = crossCombine(
            range(numVertexAttribs).map(attrib_id => ({ attrib_id })),
            FUNCS
        );

        const actualResults = [];
        const expectedResults = [];

        for (const test of TESTS) {
            const out_vals = new test.array_ctor(4);
            if (test.array_ctor === Float32Array) {
                for (let i = 0; i < 4; i++) {
                    const f01 = drng.next_unorm();
                    out_vals[i] = (2 * f01 - 1) * 1_000_000;
                }
            } else {
                for (let i = 0; i < 4; i++) {
                    out_vals[i] = drng.next_u32();
                }
            }

            const in_vals = out_vals.slice(0, test.val_count);
            const DEFAULT_VALUES = [0, 0, 0, 1];
            for (let i = 0; i < 4; i++) {
                if (i >= test.val_count) {
                    out_vals[i] = DEFAULT_VALUES[i];
                }
            }

            let args;
            if (test.func_name.endsWith('fv') || test.func_name.endsWith('iv')) {
                args = [test.attrib_id, in_vals];
            } else {
                args = [test.attrib_id, ...in_vals];
            }

            gl[test.func_name](...args);

            const attrib = gl.getVertexAttrib(test.attrib_id, gl.CURRENT_VERTEX_ATTRIB);

            actualResults.push({
                attrib_id: test.attrib_id,
                func: test.func_name,
                type: attrib?.constructor?.name,
                values: Array.from(attrib || [])
            });

            expectedResults.push({
                attrib_id: test.attrib_id,
                func: test.func_name,
                type: test.array_ctor.name,
                values: Array.from(out_vals)
            });
        }

        assert.deepStrictEqual(actualResults, expectedResults);
        checkError(gl.NO_ERROR);
    });
});
