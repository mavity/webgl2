
import fs from 'fs';
import path from 'path';
import { test } from 'node:test';
import assert from 'node:assert';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// Load the WASM module
const wasmPath = path.join(__dirname, '../webgl2.wasm');
const wasmBuffer = fs.readFileSync(wasmPath);

test('Check Attribute Locations', async (t) => {
    let ctx;
    const wasmModule = await WebAssembly.compile(wasmBuffer);
    let instance;
    const importObject = {
        env: {
            print: (ptr, len) => {
                const mem = new Uint8Array(instance.exports.memory.buffer);
                const bytes = mem.subarray(ptr, ptr + len);
                console.log(new TextDecoder('utf-8').decode(bytes));
            },
            wasm_execute_shader: () => {},
            wasm_register_shader: () => { return 0; },
            wasm_release_shader_index: () => {},
            wasm_sync_turbo_globals: () => {},
            dispatch_uncaptured_error: () => {},
            ACTIVE_ATTR_PTR: new WebAssembly.Global({ value: 'i32', mutable: true }, 0),
            ACTIVE_UNIFORM_PTR: new WebAssembly.Global({ value: 'i32', mutable: true }, 0),
            ACTIVE_VARYING_PTR: new WebAssembly.Global({ value: 'i32', mutable: true }, 0),
            ACTIVE_PRIVATE_PTR: new WebAssembly.Global({ value: 'i32', mutable: true }, 0),
            ACTIVE_TEXTURE_PTR: new WebAssembly.Global({ value: 'i32', mutable: true }, 0),
            ACTIVE_FRAME_SP: new WebAssembly.Global({ value: 'i32', mutable: true }, 0),
            // Required by egg crate for timing measurements
            now: () => {
                return performance.now();
            },
            __indirect_function_table: new WebAssembly.Table({ initial: 4096, element: 'anyfunc' })
        },
        math: {
            sin: Math.sin,
            cos: Math.cos,
            tan: Math.tan,
            asin: Math.asin,
            acos: Math.acos,
            atan: Math.atan,
            atan2: Math.atan2,
            exp: Math.exp,
            exp2: (x) => Math.pow(2, x),
            log: Math.log,
            log2: Math.log2,
            pow: Math.pow
        }
    };
    instance = await WebAssembly.instantiate(wasmModule, importObject);

    // Initialize context
    const handle = instance.exports.wasm_create_context_with_flags(1, 640, 480);
    
    // Helper to call WASM functions
    const gl = {
        createShader: (type) => instance.exports.wasm_ctx_create_shader(handle, type),
        shaderSource: (shader, src) => {
            const ptr = instance.exports.wasm_alloc(src.length + 1);
            const mem = new Uint8Array(instance.exports.memory.buffer);
            new TextEncoder().encodeInto(src, mem.subarray(ptr));
            mem[ptr + src.length] = 0;
            instance.exports.wasm_ctx_shader_source(handle, shader, ptr, src.length);
            instance.exports.wasm_free(ptr);
        },
        compileShader: (shader) => instance.exports.wasm_ctx_compile_shader(handle, shader),
        createProgram: () => instance.exports.wasm_ctx_create_program(handle),
        attachShader: (program, shader) => instance.exports.wasm_ctx_attach_shader(handle, program, shader),
        linkProgram: (program) => instance.exports.wasm_ctx_link_program(handle, program),
        getAttribLocation: (program, name) => {
            const ptr = instance.exports.wasm_alloc(name.length + 1);
            const mem = new Uint8Array(instance.exports.memory.buffer);
            new TextEncoder().encodeInto(name, mem.subarray(ptr));
            mem[ptr + name.length] = 0;
            const loc = instance.exports.wasm_ctx_get_attrib_location(handle, program, ptr, name.length);
            instance.exports.wasm_free(ptr);
            return loc;
        }
    };

    const vsSrc = `#version 300 es
    precision highp float;
    precision highp int;
    layout(location=0) in ivec4 a_val;
    layout(location=1) in uvec4 a_uval;
    flat out ivec4 v_val;
    flat out uvec4 v_uval;
    void main() {
        v_val = a_val;
        v_uval = a_uval;
        gl_Position = vec4(0, 0, 0, 1);
    }`;

    const fsSrc = `#version 300 es
    precision highp float;
    out vec4 fragColor;
    void main() {
        fragColor = vec4(1, 0, 0, 1);
    }`;

    const vs = gl.createShader(0x8B31); // VERTEX_SHADER
    gl.shaderSource(vs, vsSrc);
    gl.compileShader(vs);

    const fs = gl.createShader(0x8B30); // FRAGMENT_SHADER
    gl.shaderSource(fs, fsSrc);
    gl.compileShader(fs);

    const prog = gl.createProgram();
    gl.attachShader(prog, vs);
    gl.attachShader(prog, fs);
    gl.linkProgram(prog);

    const loc0 = gl.getAttribLocation(prog, "a_val");
    const loc1 = gl.getAttribLocation(prog, "a_uval");

    assert.deepStrictEqual({ loc0, loc1 }, { loc0: 0, loc1: 1 }, "Attribute locations should match");
});
