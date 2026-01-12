import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2, getShaderWat } from '../index.js';

test('Texture helper emission', async (t) => {
    await t.test('Helper NOT emitted for shader without textures', async () => {
        const gl = await webGL2();
        try {
            const vs = gl.createShader(gl.VERTEX_SHADER);
            gl.shaderSource(vs, `#version 300 es
            void main() { gl_Position = vec4(0.0); }`);
            gl.compileShader(vs);

            const fs = gl.createShader(gl.FRAGMENT_SHADER);
            gl.shaderSource(fs, `#version 300 es
            precision mediump float; out vec4 color;
            void main() { color = vec4(1.0); }`);
            gl.compileShader(fs);

            const program = gl.createProgram();
            gl.attachShader(program, vs);
            gl.attachShader(program, fs);
            gl.linkProgram(program);
            assert.strictEqual(gl.getProgramParameter(program, gl.LINK_STATUS), true);

            const fsWat = getShaderWat(gl._ctxHandle, program._handle, gl.FRAGMENT_SHADER);
            assert.ok(fsWat, 'VAT should be returned');
            assert.doesNotMatch(fsWat, /func \$__webgl_texture_sample/, 'Helper should NOT be present in non-texture FS');
        } finally {
            gl.destroy();
        }
    });

    await t.test('Helper IS emitted for shader with textures', async () => {
        const gl = await webGL2();
        try {
            const vs = gl.createShader(gl.VERTEX_SHADER);
            gl.shaderSource(vs, `#version 300 es
            out vec2 v_uv;
            void main() { v_uv = vec2(0.5); gl_Position = vec4(0.0); }`);
            gl.compileShader(vs);

            const fs = gl.createShader(gl.FRAGMENT_SHADER);
            gl.shaderSource(fs, `#version 300 es
            precision mediump float;
            uniform texture2D u_tex;
            uniform sampler u_samp;
            in vec2 v_uv;
            out vec4 color;
            void main() { 
                color = texture(sampler2D(u_tex, u_samp), v_uv); 
            }`);
            gl.compileShader(fs);
            
            if (!gl.getShaderParameter(fs, gl.COMPILE_STATUS)) {
                throw new Error("FS compile failed: " + gl.getShaderInfoLog(fs));
            }

            const program = gl.createProgram();
            gl.attachShader(program, vs);
            gl.attachShader(program, fs);
            gl.linkProgram(program);
            
            if (!gl.getProgramParameter(program, gl.LINK_STATUS)) {
                throw new Error("Link failed: " + gl.getProgramInfoLog(program));
            }

            const fsWat = getShaderWat(gl._ctxHandle, program._handle, gl.FRAGMENT_SHADER);
            assert.match(fsWat, /func \$__webgl_texture_sample/, 'Helper should be present in texture FS');
            
        } finally {
            gl.destroy();
        }
    });
});
