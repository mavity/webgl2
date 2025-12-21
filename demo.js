// Environment detection
const isNode = typeof process !== 'undefined' && process.versions && process.versions.node;

// Import fs only in Node
let fs;
async function initFS() {
    if (isNode) {
        const fsModule = await import('fs');
        fs = fsModule.default;
    }
}

async function renderCube() {
    let webGL2;
    try {
        const { webGL2: webGL2Loaded } = await import(
            typeof loadModulePath === 'string' ?
                loadModulePath :
                './index.js');
        webGL2 = webGL2Loaded;
    } catch {
        const { webGL2: webGL2Loaded } = await import('https://esm.run/webgl2');
        webGL2 = webGL2Loaded;
    }

    const gl = await webGL2();
    gl.verbosity = 0; // Disable debug logs for this demo
    gl.viewport(0, 0, 640, 480);
    
    // Shaders
    const vsSource = `#version 300 es
    layout(location = 0) in vec3 position;
    layout(location = 1) in vec2 uv;
    
    uniform mat4 u_mvp;
    
    out vec2 v_uv;
    
    void main() {
        v_uv = uv;
        gl_Position = u_mvp * vec4(position, 1.0);
    }
    `;
    
    const fsSource = `#version 300 es
    precision highp float;
    uniform texture2D u_texture;
    uniform sampler u_sampler;
    in vec2 v_uv;
    out vec4 fragColor;
    
    void main() {
        fragColor = texture(sampler2D(u_texture, u_sampler), v_uv);
        // fragColor = vec4(v_uv, 0.0, 1.0);
    }
    `;
    
    const vs = gl.createShader(gl.VERTEX_SHADER);
    gl.shaderSource(vs, vsSource);
    gl.compileShader(vs);
    
    const fsShader = gl.createShader(gl.FRAGMENT_SHADER);
    gl.shaderSource(fsShader, fsSource);
    gl.compileShader(fsShader);
    
    const program = gl.createProgram();
    gl.attachShader(program, vs);
    gl.attachShader(program, fsShader);
    gl.linkProgram(program);
    gl.useProgram(program);
    
    // Cube data
    const vertices = new Float32Array([
        // Front face
        -0.5, -0.5,  0.5,  0.0, 0.0,
         0.5, -0.5,  0.5,  1.0, 0.0,
         0.5,  0.5,  0.5,  1.0, 1.0,
        -0.5, -0.5,  0.5,  0.0, 0.0,
         0.5,  0.5,  0.5,  1.0, 1.0,
        -0.5,  0.5,  0.5,  0.0, 1.0,
        
        // Back face
        -0.5, -0.5, -0.5,  0.0, 0.0,
        -0.5,  0.5, -0.5,  0.0, 1.0,
         0.5,  0.5, -0.5,  1.0, 1.0,
        -0.5, -0.5, -0.5,  0.0, 0.0,
         0.5,  0.5, -0.5,  1.0, 1.0,
         0.5, -0.5, -0.5,  1.0, 0.0,
         
        // Top face
        -0.5,  0.5, -0.5,  0.0, 0.0,
        -0.5,  0.5,  0.5,  0.0, 1.0,
         0.5,  0.5,  0.5,  1.0, 1.0,
        -0.5,  0.5, -0.5,  0.0, 0.0,
         0.5,  0.5,  0.5,  1.0, 1.0,
         0.5,  0.5, -0.5,  1.0, 0.0,
         
        // Bottom face
        -0.5, -0.5, -0.5,  0.0, 0.0,
         0.5, -0.5, -0.5,  1.0, 0.0,
         0.5, -0.5,  0.5,  1.0, 1.0,
        -0.5, -0.5, -0.5,  0.0, 0.0,
         0.5, -0.5,  0.5,  1.0, 1.0,
        -0.5, -0.5,  0.5,  0.0, 1.0,
        
        // Right face
         0.5, -0.5, -0.5,  0.0, 0.0,
         0.5,  0.5, -0.5,  0.0, 1.0,
         0.5,  0.5,  0.5,  1.0, 1.0,
         0.5, -0.5, -0.5,  0.0, 0.0,
         0.5,  0.5,  0.5,  1.0, 1.0,
         0.5, -0.5,  0.5,  1.0, 0.0,
         
        // Left face
        -0.5, -0.5, -0.5,  0.0, 0.0,
        -0.5, -0.5,  0.5,  1.0, 0.0,
        -0.5,  0.5,  0.5,  1.0, 1.0,
        -0.5, -0.5, -0.5,  0.0, 0.0,
        -0.5,  0.5,  0.5,  1.0, 1.0,
        -0.5,  0.5, -0.5,  0.0, 1.0,
    ]);
    
    const buffer = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, buffer);
    gl.bufferData(gl.ARRAY_BUFFER, vertices, gl.STATIC_DRAW);
    
    gl.enableVertexAttribArray(0);
    gl.vertexAttribPointer(0, 3, gl.FLOAT, false, 20, 0);
    gl.enableVertexAttribArray(1);
    gl.vertexAttribPointer(1, 2, gl.FLOAT, false, 20, 12);
    
    // Texture
    const tex = gl.createTexture();
    gl.bindTexture(gl.TEXTURE_2D, tex);
    const texData = new Uint8Array(16 * 16 * 4);
    for (let y = 0; y < 16; y++) {
        for (let x = 0; x < 16; x++) {
            const idx = (y * 16 + x) * 4;
            const isCheck = ((x >> 2) ^ (y >> 2)) & 1;
            if (isCheck) {
                texData[idx] = 255; texData[idx+1] = 215; texData[idx+2] = 0; texData[idx+3] = 255; // Gold
            } else {
                texData[idx] = 100; texData[idx+1] = 149; texData[idx+2] = 237; texData[idx+3] = 255; // CornflowerBlue
            }
        }
    }
    gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, 16, 16, 0, gl.RGBA, gl.UNSIGNED_BYTE, texData);
    
    const uTextureLoc = gl.getUniformLocation(program, "u_texture");
    gl.uniform1i(uTextureLoc, 0);
    const uSamplerLoc = gl.getUniformLocation(program, "u_sampler");
    gl.uniform1i(uSamplerLoc, 0);
    
    // Matrix math
    function perspective(fovy, aspect, near, far) {
        const f = 1.0 / Math.tan(fovy / 2);
        const nf = 1 / (near - far);
        return [
            f / aspect, 0, 0, 0,
            0, f, 0, 0,
            0, 0, (far + near) * nf, -1,
            0, 0, (2 * far * near) * nf, 0
        ];
    }
    
    function multiply(a, b) {
        const out = new Float32Array(16);
        for (let col = 0; col < 4; col++) {
            for (let row = 0; row < 4; row++) {
                let sum = 0;
                for (let k = 0; k < 4; k++) {
                    sum += a[k * 4 + row] * b[col * 4 + k];
                }
                out[col * 4 + row] = sum;
            }
        }
        return out;
    }
    
    function rotateY(m, angle) {
        const c = Math.cos(angle);
        const s = Math.sin(angle);
        const r = [
            c, 0, -s, 0,
            0, 1, 0, 0,
            s, 0, c, 0,
            0, 0, 0, 1
        ];
        return multiply(m, r);
    }

    function rotateX(m, angle) {
        const c = Math.cos(angle);
        const s = Math.sin(angle);
        const r = [
            1, 0, 0, 0,
            0, c, s, 0,
            0, -s, c, 0,
            0, 0, 0, 1
        ];
        return multiply(m, r);
    }
    
    function translate(m, x, y, z) {
        const t = [
            1, 0, 0, 0,
            0, 1, 0, 0,
            0, 0, 1, 0,
            x, y, z, 1
        ];
        return multiply(m, t);
    }
    
    let mvp = perspective(Math.PI / 4, 640 / 480, 0.1, 100.0);
    mvp = translate(mvp, 0, 0, -3);
    mvp = rotateX(mvp, 0.5);
    mvp = rotateY(mvp, 0.8);
    
    const mvpLoc = gl.getUniformLocation(program, "u_mvp");
    gl.uniformMatrix4fv(mvpLoc, false, mvp);
    console.log("MVP Matrix:", mvp);
    
    // Render
    gl.clearColor(0.0, 0.0, 0.0, 0.0);
    gl.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT);
    gl.drawArrays(gl.TRIANGLES, 0, 36);
    
    // Read pixels
    const pixels = new Uint8Array(640 * 480 * 4);
    gl.readPixels(0, 0, 640, 480, gl.RGBA, gl.UNSIGNED_BYTE, pixels);
    
    return { pixels, width: 640, height: 480 };
}

// Matrix from PNG rendering functions
function savePNG(width, height, pixels, filename) {
    // PNG Signature
    const signature = Buffer.from([0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]);

    function createChunk(type, data) {
        const len = Buffer.alloc(4);
        len.writeUInt32BE(data.length, 0);
        const typeBuf = Buffer.from(type);
        const crc = Buffer.alloc(4);
        crc.writeUInt32BE(crc32(Buffer.concat([typeBuf, data])), 0);
        return Buffer.concat([len, typeBuf, data, crc]);
    }

    // IHDR
    const ihdrData = Buffer.alloc(13);
    ihdrData.writeUInt32BE(width, 0);
    ihdrData.writeUInt32BE(height, 4);
    ihdrData[8] = 8; // bit depth
    ihdrData[9] = 6; // color type (RGBA)
    ihdrData[10] = 0; // compression
    ihdrData[11] = 0; // filter
    ihdrData[12] = 0; // interlace
    const ihdr = createChunk('IHDR', ihdrData);

    // IDAT (ZLIB uncompressed)
    // Each scanline starts with a filter byte (0)
    const scanlineSize = width * 4 + 1;
    const uncompressedData = Buffer.alloc(height * scanlineSize);
    for (let y = 0; y < height; y++) {
        const srcY = height - 1 - y; // Flip Y for PNG (gl.readPixels is bottom-up)
        uncompressedData[y * scanlineSize] = 0; // Filter None
        pixels.copy(uncompressedData, y * scanlineSize + 1, srcY * width * 4, (srcY + 1) * width * 4);
    }

    // ZLIB wrap
    const zlibHeader = Buffer.from([0x78, 0x01]);
    const blocks = [];
    for (let i = 0; i < uncompressedData.length; i += 65535) {
        const remaining = uncompressedData.length - i;
        const blockSize = Math.min(remaining, 65535);
        const isLast = remaining <= 65535;
        const blockHeader = Buffer.alloc(5);
        blockHeader[0] = isLast ? 1 : 0;
        blockHeader.writeUInt16LE(blockSize, 1);
        blockHeader.writeUInt16LE(~blockSize & 0xFFFF, 3);
        blocks.push(blockHeader);
        blocks.push(uncompressedData.slice(i, i + blockSize));
    }
    const adler = Buffer.alloc(4);
    adler.writeUInt32BE(adler32(uncompressedData), 0);
    const idatData = Buffer.concat([zlibHeader, ...blocks, adler]);
    const idat = createChunk('IDAT', idatData);

    // IEND
    const iend = createChunk('IEND', Buffer.alloc(0));

    fs.writeFileSync(filename, Buffer.concat([signature, ihdr, idat, iend]));
}

// CRC32 implementation
const crcTable = new Uint32Array(256);
for (let i = 0; i < 256; i++) {
    let c = i;
    for (let j = 0; j < 8; j++) {
        c = (c & 1) ? (0xEDB88320 ^ (c >>> 1)) : (c >>> 1);
    }
    crcTable[i] = c;
}

function crc32(buf) {
    let crc = 0xFFFFFFFF;
    for (let i = 0; i < buf.length; i++) {
        crc = crcTable[(crc ^ buf[i]) & 0xFF] ^ (crc >>> 8);
    }
    return (crc ^ 0xFFFFFFFF) >>> 0;
}

function adler32(buf) {
    let s1 = 1, s2 = 0;
    for (let i = 0; i < buf.length; i++) {
        s1 = (s1 + buf[i]) % 65521;
        s2 = (s2 + s1) % 65521;
    }
    return ((s2 << 16) | s1) >>> 0;
}

// Main entry point
async function main() {
    await initFS();
    const result = await renderCube();
    const { pixels, width, height } = result;
    
    if (isNode) {
        // Node: Save to file
        savePNG(width, height, Buffer.from(pixels), 'output.png');
        console.log("Saved output.png");
    } else {
        // Browser: Apply styles and create UI
        const style = document.createElement('style');
        style.textContent = `
            * { margin: 0; padding: 0; box-sizing: border-box; }
            body {
                background: #222;
                color: #fff;
                font-family: monospace;
                display: flex;
                flex-direction: column;
                align-items: center;
                justify-content: center;
                min-height: 100vh;
                padding: 20px;
                gap: 20px;
            }
            h1 {
                font-size: 24px;
                font-weight: normal;
            }
            canvas {
                border: 2px solid #fff;
                background: #000;
                image-rendering: pixelated;
                image-rendering: crisp-edges;
            }
        `;
        document.head.appendChild(style);
        
        // Create title
        const h1 = document.createElement('h1');
        h1.textContent = 'WebGL2 Polymorphic Cube Renderer';
        document.body.appendChild(h1);
        
        // Create canvas
        const canvas = document.createElement('canvas');
        canvas.width = width;
        canvas.height = height;
        document.body.appendChild(canvas);
        
        const ctx = canvas.getContext('2d');
        const imageData = ctx.createImageData(width, height);
        
        // Flip Y axis: gl.readPixels is bottom-up, canvas is top-down
        const flipped = new Uint8ClampedArray(pixels.length);
        for (let y = 0; y < height; y++) {
            const srcY = height - 1 - y;
            const srcOffset = srcY * width * 4;
            const dstOffset = y * width * 4;
            flipped.set(pixels.subarray(srcOffset, srcOffset + width * 4), dstOffset);
        }
        
        imageData.data.set(flipped);
        ctx.putImageData(imageData, 0, 0);
        
        console.log("Rendered cube to canvas");
    }
}

// Auto-run if this is Node
main().catch(console.error);

// Export for browser
export { renderCube, main };
