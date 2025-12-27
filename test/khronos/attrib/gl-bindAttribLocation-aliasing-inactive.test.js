import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../../../index.js';

// Helper to replace params like $(var)
function replaceParams(str, replacements) {
    let result = str;
    for (const [key, value] of Object.entries(replacements)) {
        result = result.split(`$(${key})`).join(value);
    }
    return result;
}

test('bindAttribLocation with aliasing - inactive attributes', async (t) => {
    const gl = await webGL2();

    // Helper to compile shader
    function loadShader(gl, source, type) {
        const shader = gl.createShader(type);
        gl.shaderSource(shader, source);
        gl.compileShader(shader);
        if (!gl.getShaderParameter(shader, gl.COMPILE_STATUS)) {
            console.error(gl.getShaderInfoLog(shader));
            return null;
        }
        return shader;
    }

    try {
        const simpleColorFragmentShaderESSL300 = `#version 300 es
            precision mediump float;
            out vec4 my_FragColor;
            void main() {
                my_FragColor = vec4(0.0, 1.0, 0.0, 1.0);
            }
        `;

        const glFragmentShader = loadShader(gl, simpleColorFragmentShaderESSL300, gl.FRAGMENT_SHADER);
        assert.ok(glFragmentShader, "Fragment shader compiled successfully");

        const typeInfo = [
            { type: 'float', asVec4: 'vec4(0.0, $(var), 0.0, 1.0)' },
            { type: 'vec2', asVec4: 'vec4($(var), 0.0, 1.0)' },
            { type: 'vec3', asVec4: 'vec4($(var), 1.0)' },
            { type: 'vec4', asVec4: '$(var)' },
        ];

        const maxAttributes = gl.getParameter(gl.MAX_VERTEX_ATTRIBS);

        const runTest = (vertexShaderTemplate, description) => {
            // Test all type combinations of a_1 and a_2.
            typeInfo.forEach(function (typeInfo1) {
                typeInfo.forEach(function (typeInfo2) {
                    var replaceParamsMap = {
                        type_1: typeInfo1.type,
                        type_2: typeInfo2.type,
                        gl_Position_1: replaceParams(typeInfo1.asVec4, { var: 'a_1' }),
                        gl_Position_2: replaceParams(typeInfo2.asVec4, { var: 'a_2' })
                    };
                    var strVertexShader = replaceParams(vertexShaderTemplate, replaceParamsMap);
                    var glVertexShader = loadShader(gl, strVertexShader, gl.VERTEX_SHADER);
                    assert.ok(glVertexShader, `Vertex shader compiled successfully for ${description} (${typeInfo1.type}, ${typeInfo2.type})`);

                    // Bind both a_1 and a_2 to the same position and verify the link fails.
                    // Do so for all valid positions available.
                    for (var l = 0; l < maxAttributes; l++) {
                        var glProgram = gl.createProgram();
                        gl.bindAttribLocation(glProgram, l, 'a_1');
                        gl.bindAttribLocation(glProgram, l, 'a_2');
                        gl.attachShader(glProgram, glVertexShader);
                        gl.attachShader(glProgram, glFragmentShader);
                        gl.linkProgram(glProgram);
                        var linkStatus = gl.getProgramParameter(glProgram, gl.LINK_STATUS);
                        assert.equal(linkStatus, false, `Link should fail when both attributes are aliased to location ${l}. Case: ${description} (${typeInfo1.type}, ${typeInfo2.type})`);

                        // Cleanup program
                        gl.deleteProgram(glProgram);
                    }
                    gl.deleteShader(glVertexShader);
                });
            });
        };

        const vertexShaderStaticallyUsedButInactive = `#version 300 es
            precision mediump float;
            in $(type_1) a_1;
            in $(type_2) a_2;
            void main() {
                gl_Position = true ? $(gl_Position_1) : $(gl_Position_2);
            }
        `;

        const vertexShaderUnused = `#version 300 es
            precision mediump float;
            in $(type_1) a_1;
            in $(type_2) a_2;
            void main() {
                gl_Position = vec4(0.0);
            }
        `;

        runTest(vertexShaderStaticallyUsedButInactive, "Statically used but inactive");
        runTest(vertexShaderUnused, "Entirely unused");

    } finally {
        gl.destroy();
    }
});
