import { webGL2 } from '../index.js';

(async function(){
  const gl = await webGL2();
  try {
    gl.viewport(0,0,1,1);
    const VS = `#version 300 es
    layout(location=0) in vec2 a_pos;
    layout(location=1) in vec2 a_uv;
    out vec2 v_uv;
    void main() { v_uv = a_uv; gl_Position = vec4(a_pos,0.0,1.0); }
    `;
    const FS = `#version 300 es
    precision highp float;
    in vec2 v_uv; out vec4 fragColor;
    void main() { fragColor = vec4(v_uv, 0.0, 1.0); }
    `;

    const compileProgram = async (vsSource, fsSource) => {
      const vs = gl.createShader(gl.VERTEX_SHADER);
      gl.shaderSource(vs, vsSource);
      gl.compileShader(vs);
      const fs = gl.createShader(gl.FRAGMENT_SHADER);
      gl.shaderSource(fs, fsSource);
      gl.compileShader(fs);
      const program = gl.createProgram();
      gl.attachShader(program, vs);
      gl.attachShader(program, fs);
      gl.linkProgram(program);
      return program;
    };

    const program = await compileProgram(VS, FS);
    gl.useProgram(program);

    const vertices = new Float32Array([
      -1, -1, 0, 0,
      1, -1, 1, 0,
      1, 1, 1, 1,
      -1, -1, 0, 0,
      1, 1, 1, 1,
      -1, 1, 0, 1,
    ]);
    const buf = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, buf);
    gl.bufferData(gl.ARRAY_BUFFER, vertices, gl.STATIC_DRAW);
    gl.enableVertexAttribArray(0);
    gl.vertexAttribPointer(0, 2, gl.FLOAT, false, 16, 0);
    gl.enableVertexAttribArray(1);
    gl.vertexAttribPointer(1, 2, gl.FLOAT, false, 16, 8);

    gl.clearColor(0,0,0,0);
    gl.clear(gl.COLOR_BUFFER_BIT);
    gl.drawArrays(gl.TRIANGLES, 0, 6);

    const out = new Float32Array(4);
    gl.readPixels(0,0,1,1,0x1908,0x1406,out); // GL_RGBA, GL_FLOAT
    console.log('read v_uv (r,g):', out[0], out[1], 'a:', out[3]);
  } finally {
    gl.destroy();
  }
})();
