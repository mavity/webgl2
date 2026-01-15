// Environment detection
const isNode = typeof process !== 'undefined' && process.versions && process.versions.node;

// Animation state for browser
const animationState = {
    running: false,
    frameCount: 0,
    lastFpsTime: Date.now(),
    fps: 0,
    fpsElement: null,
    button: null,
    canvas: null,
    ctx: null,
    width: 0,
    height: 0,
    startTime: null
};

// Global rendering context (lazy initialized)
let renderContext = null;

async function initializeRenderContext() {
    if (renderContext) return renderContext;

    let loadLocal =
        isNode || (
            typeof location !== 'undefined' &&
            typeof location?.hostname === 'string' &&
            (location.hostname.toString() === 'localhost' || location.hostname.toString() === '127.0.0.1')
        );
    const { webGL2 } = await import(
        loadLocal ? './index.js' :
            'https://esm.run/webgl2'
    );

    const gl = await webGL2({ debug: true });
    gl.viewport(0, 0, 640, 480);

    // Shaders
    const vsSource = /* glsl */`#version 300 es
    layout(location = 0) in vec3 position;
    layout(location = 1) in vec2 uv;
    
    uniform mat4 u_mvp;
    
    out vec2 v_uv;
    
    void main() {
        v_uv = uv;
        gl_Position = u_mvp * vec4(position, 1.0);
    }
    `;

    const fsSource = /* glsl */`#version 300 es
    precision highp float;
    uniform texture2D u_texture;
    uniform sampler u_sampler;
    in vec2 v_uv;
    out vec4 fragColor;

    void small_fn_before(float val_noop) {
        val_noop = 1.0;
    }

    void small_fn_after(float val_noop) {
        val_noop = 2.0;
    }

    void main() {
        small_fn_before(3.0);
        small_fn_after(4.0);
        fragColor = texture(sampler2D(u_texture, u_sampler), v_uv);
        // small_fn_after(4.0);
        // fragColor = vec4(v_uv, 0.0, 1.0);
    }`;

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
        -0.5, -0.5, 0.5, 0.0, 0.0,
        0.5, -0.5, 0.5, 1.0, 0.0,
        0.5, 0.5, 0.5, 1.0, 1.0,
        -0.5, -0.5, 0.5, 0.0, 0.0,
        0.5, 0.5, 0.5, 1.0, 1.0,
        -0.5, 0.5, 0.5, 0.0, 1.0,

        // Back face
        -0.5, -0.5, -0.5, 0.0, 0.0,
        -0.5, 0.5, -0.5, 0.0, 1.0,
        0.5, 0.5, -0.5, 1.0, 1.0,
        -0.5, -0.5, -0.5, 0.0, 0.0,
        0.5, 0.5, -0.5, 1.0, 1.0,
        0.5, -0.5, -0.5, 1.0, 0.0,

        // Top face
        -0.5, 0.5, -0.5, 0.0, 0.0,
        -0.5, 0.5, 0.5, 0.0, 1.0,
        0.5, 0.5, 0.5, 1.0, 1.0,
        -0.5, 0.5, -0.5, 0.0, 0.0,
        0.5, 0.5, 0.5, 1.0, 1.0,
        0.5, 0.5, -0.5, 1.0, 0.0,

        // Bottom face
        -0.5, -0.5, -0.5, 0.0, 0.0,
        0.5, -0.5, -0.5, 1.0, 0.0,
        0.5, -0.5, 0.5, 1.0, 1.0,
        -0.5, -0.5, -0.5, 0.0, 0.0,
        0.5, -0.5, 0.5, 1.0, 1.0,
        -0.5, -0.5, 0.5, 0.0, 1.0,

        // Right face
        0.5, -0.5, -0.5, 0.0, 0.0,
        0.5, 0.5, -0.5, 0.0, 1.0,
        0.5, 0.5, 0.5, 1.0, 1.0,
        0.5, -0.5, -0.5, 0.0, 0.0,
        0.5, 0.5, 0.5, 1.0, 1.0,
        0.5, -0.5, 0.5, 1.0, 0.0,

        // Left face
        -0.5, -0.5, -0.5, 0.0, 0.0,
        -0.5, -0.5, 0.5, 1.0, 0.0,
        -0.5, 0.5, 0.5, 1.0, 1.0,
        -0.5, -0.5, -0.5, 0.0, 0.0,
        -0.5, 0.5, 0.5, 1.0, 1.0,
        -0.5, 0.5, -0.5, 0.0, 1.0,
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
                texData[idx] = 255; texData[idx + 1] = 215; texData[idx + 2] = 0; texData[idx + 3] = 255; // Gold
            } else {
                texData[idx] = 100; texData[idx + 1] = 149; texData[idx + 2] = 237; texData[idx + 3] = 255; // CornflowerBlue
            }
        }
    }
    gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, 16, 16, 0, gl.RGBA, gl.UNSIGNED_BYTE, texData);

    const uTextureLoc = gl.getUniformLocation(program, "u_texture");
    gl.uniform1i(uTextureLoc, 0);
    const uSamplerLoc = gl.getUniformLocation(program, "u_sampler");
    gl.uniform1i(uSamplerLoc, 0);

    // Matrix math functions
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

    // Calculate initial MVP matrix
    let mvp = perspective(Math.PI / 4, 640 / 480, 0.1, 100.0);
    mvp = translate(mvp, 0, 0, -3);
    mvp = rotateX(mvp, 0.5);
    mvp = rotateY(mvp, 0.8);

    const mvpLoc = gl.getUniformLocation(program, "u_mvp");

    renderContext = {
        gl,
        program,
        mvpLoc,
        mvp: new Float32Array(mvp),
        // Store functions and base values for dynamic rotation
        perspective,
        translate,
        rotateX,
        rotateY,
        multiply
    };

    return renderContext;
}

async function renderCube(elapsedTime = 0) {
    const ctx = await initializeRenderContext();
    const { gl, program, mvpLoc, perspective, translate, rotateX, rotateY, multiply } = ctx;

    // Calculate rotation angle: 1 full rotation (2π) in 5 seconds
    const rotationAngle = (elapsedTime / 5000) * Math.PI * 2;

    // Recalculate MVP with time-based rotation
    let mvp = perspective(Math.PI / 4, 640 / 480, 0.1, 100.0);
    mvp = translate(mvp, 0, 0, -3);
    mvp = rotateX(mvp, 0.5);
    mvp = rotateY(mvp, 0.8 + rotationAngle);

    // Set MVP matrix
    gl.uniformMatrix4fv(mvpLoc, false, mvp);
    // console.log("MVP Matrix:", mvp);

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
function createPNG(width, height, pixels) {
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

    const buf = Buffer.concat([signature, ihdr, idat, iend]);
    return buf;
}

// ASCII art rendering function for terminal
function renderASCIIArt(pixels, width, height) {
    // Step 1: Crop - horizontal: 1/6 from each side, vertical: 1/5 from top, 2/15 from bottom
    const cropX = Math.floor(width / 6);
    const cropTop = Math.floor(height / 5);
    const cropBottom = Math.floor(height * 2 / 15);
    const croppedWidth = width - 2 * cropX;
    const croppedHeight = height - cropTop - cropBottom;

    // Step 2: Calculate target dimensions (80 chars wide, proportional height)
    const targetWidth = 80;
    const targetHeight = Math.floor((croppedHeight / croppedWidth) * targetWidth);

    // Step 3: Downsample the image
    // Each character represents 2 vertical pixels
    const charHeight = Math.floor(targetHeight / 2);

    const sampledPixels = [];
    for (let charY = 0; charY < charHeight; charY++) {
        for (let charX = 0; charX < targetWidth; charX++) {
            // Sample two pixels: top and bottom for this character
            const topPixel = samplePixel(pixels, width, height, cropX, cropTop, cropBottom, croppedWidth, croppedHeight, charX, charY * 2, targetWidth, targetHeight);
            const bottomPixel = samplePixel(pixels, width, height, cropX, cropTop, cropBottom, croppedWidth, croppedHeight, charX, charY * 2 + 1, targetWidth, targetHeight);
            sampledPixels.push({ top: topPixel, bottom: bottomPixel });
        }
    }

    // Step 4: Generate ASCII art with ANSI colors
    let output = '';
    for (let charY = 0; charY < charHeight; charY++) {
        for (let charX = 0; charX < targetWidth; charX++) {
            const idx = charY * targetWidth + charX;
            const { top, bottom } = sampledPixels[idx];

            // Determine if pixels are blue, yellow, or something else
            const topColor = classifyColor(top);
            const bottomColor = classifyColor(bottom);

            // Generate character with ANSI codes
            const charData = generateChar(topColor, bottomColor);
            output += charData;
        }
        output += '\n';
    }

    return output;
}

function samplePixel(pixels, origWidth, origHeight, cropX, cropTop, cropBottom, croppedWidth, croppedHeight, x, y, targetWidth, targetHeight) {
    // Map from target coordinates to cropped source coordinates
    const srcX = Math.floor((x / targetWidth) * croppedWidth) + cropX;
    const srcY = Math.floor((y / targetHeight) * croppedHeight) + cropTop;

    // Flip Y coordinate (pixels are bottom-up from readPixels)
    const flippedY = origHeight - 1 - srcY;

    const idx = (flippedY * origWidth + srcX) * 4;
    return {
        r: pixels[idx],
        g: pixels[idx + 1],
        b: pixels[idx + 2],
        a: pixels[idx + 3]
    };
}

function classifyColor(pixel) {
    // Check for transparent/black
    if (pixel.a < 128 || (pixel.r < 50 && pixel.g < 50 && pixel.b < 50)) {
        return 'black';
    }

    // Check for blue (cornflower blue: 100, 149, 237)
    const isBlue = pixel.b > pixel.r && pixel.b > pixel.g && pixel.b > 150;
    if (isBlue) {
        return 'blue';
    }

    // Check for yellow/gold (255, 215, 0)
    const isYellow = pixel.r > 200 && pixel.g > 150 && pixel.b < 100;
    if (isYellow) {
        return 'yellow';
    }

    return 'other';
}

function generateChar(topColor, bottomColor) {
    // Unicode block characters
    const FULL_BLOCK = '█';
    const UPPER_HALF_BLOCK = '▀';
    const LOWER_HALF_BLOCK = '▄';
    const SPACE = ' ';

    // ANSI color codes
    const RESET = '\x1b[0m';
    const BLUE_256 = '\x1b[38;5;69m';   // Cornflower blue approximation
    const YELLOW_256 = '\x1b[38;5;220m'; // Gold approximation
    const BLUE_16 = '\x1b[34m';          // Fallback blue
    const YELLOW_16 = '\x1b[33m';        // Fallback yellow

    // Wrap 256-color in 16-color for graceful degradation
    const BLUE = BLUE_16 + BLUE_256;
    const YELLOW = YELLOW_16 + YELLOW_256;

    // Both same color
    if (topColor === bottomColor) {
        if (topColor === 'blue') {
            return BLUE + FULL_BLOCK + RESET;
        } else if (topColor === 'yellow') {
            return YELLOW + FULL_BLOCK + RESET;
        } else {
            return SPACE;
        }
    }

    // Different colors - use half blocks
    if (topColor === 'blue' && bottomColor === 'yellow') {
        return BLUE + UPPER_HALF_BLOCK + RESET;
    } else if (topColor === 'yellow' && bottomColor === 'blue') {
        return BLUE + LOWER_HALF_BLOCK + RESET;
    } else if (topColor === 'blue' && bottomColor === 'black') {
        return BLUE + UPPER_HALF_BLOCK + RESET;
    } else if (topColor === 'black' && bottomColor === 'blue') {
        return BLUE + LOWER_HALF_BLOCK + RESET;
    } else if (topColor === 'yellow' && bottomColor === 'black') {
        return YELLOW + UPPER_HALF_BLOCK + RESET;
    } else if (topColor === 'black' && bottomColor === 'yellow') {
        return YELLOW + LOWER_HALF_BLOCK + RESET;
    } else if (topColor === 'blue') {
        return BLUE + UPPER_HALF_BLOCK + RESET;
    } else if (bottomColor === 'blue') {
        return BLUE + LOWER_HALF_BLOCK + RESET;
    } else if (topColor === 'yellow') {
        return YELLOW + UPPER_HALF_BLOCK + RESET;
    } else if (bottomColor === 'yellow') {
        return YELLOW + LOWER_HALF_BLOCK + RESET;
    }

    return SPACE;
}

// Main entry point
async function displayFrame(pixels, width, height) {
    if (!animationState.ctx) return;

    const imageData = animationState.ctx.createImageData(width, height);

    // Flip Y axis: gl.readPixels is bottom-up, canvas is top-down
    const flipped = new Uint8ClampedArray(pixels.length);
    for (let y = 0; y < height; y++) {
        const srcY = height - 1 - y;
        const srcOffset = srcY * width * 4;
        const dstOffset = y * width * 4;
        flipped.set(pixels.subarray(srcOffset, srcOffset + width * 4), dstOffset);
    }

    imageData.data.set(flipped);
    animationState.ctx.putImageData(imageData, 0, 0);
}

function updateFpsCounter() {
    const now = Date.now();
    const deltaTime = now - animationState.lastFpsTime;

    if (deltaTime >= 500) {
        animationState.fps = Math.round((animationState.frameCount * 1000) / deltaTime);
        if (animationState.fpsElement) {
            animationState.fpsElement.textContent = `FPS: ${animationState.fps}`;
        }
        animationState.frameCount = 0;
        animationState.lastFpsTime = now;
    }
}

async function animate() {
    if (!animationState.running) return;

    const elapsedTime = Date.now() - animationState.startTime;
    const result = await renderCube(elapsedTime);
    const { pixels, width, height } = result;

    await displayFrame(pixels, width, height);
    animationState.frameCount++;
    updateFpsCounter();

    requestAnimationFrame(animate);
}

// Detect ANSI support in terminal
function detectANSISupport() {
    // Check for explicit indicators of no ANSI support
    const term = process.env.TERM;
    const noColor = process.env.NO_COLOR;

    // Definitely no ANSI support
    if (term === 'dumb' || noColor !== undefined) {
        return false;
    }

    // Otherwise assume support (or ambiguous = support as per requirements)
    return true;
}

// Terminal animation loop
async function runTerminalAnimation(width, height, duration = 20000) {
    const startTime = Date.now();
    const fps = 20;
    const frameDelay = 1000 / fps;

    let firstFrame = true;
    let numLines = 0;
    let frameCount = 0;
    let lastFrameTime = startTime;

    const renderFrame = async () => {
        const now = Date.now();
        const elapsedTime = now - startTime;

        if (elapsedTime >= duration) {
            // Animation complete
            return;
        }

        // Calculate average FPS
        frameCount++;
        const avgFps = elapsedTime > 0 ? Math.round((frameCount * 1000) / elapsedTime) : 0;

        // Render cube with current rotation
        const result = await renderCube(elapsedTime);
        const { pixels } = result;

        // Generate ASCII art
        const asciiArt = renderASCIIArt(pixels, width, height);

        // Add FPS counter at the top
        const fpsLine = `FPS: ${avgFps} | Frame: ${frameCount} | Time: ${(elapsedTime / 1000).toFixed(1)}s\n`;
        const output = fpsLine + asciiArt;

        if (firstFrame) {
            // First frame: just print it
            process.stdout.write(output);
            numLines = output.split('\n').length - 1;
            firstFrame = false;
        } else {
            // Move cursor up to overwrite previous frame
            process.stdout.write(`\x1b[${numLines}A`);
            process.stdout.write(output);
        }

        lastFrameTime = now;

        // Schedule next frame
        setTimeout(renderFrame, frameDelay);
    };

    console.log("\nASCII Art Animation (20 seconds):\n");
    await renderFrame();
}

async function main() {
    const result = await renderCube();
    const { pixels, width, height } = result;

    if (isNode) {
        // Node: Save to file
        const buf = createPNG(width, height, Buffer.from(pixels));

        const fs = await import('fs');
        const path = await import('path');
        const { fileURLToPath } = await import('url');

        const __filename = fileURLToPath(import.meta.url);
        const __dirname = path.dirname(__filename);

        fs.writeFileSync(path.resolve(__dirname, 'output.png'), buf);
        console.log("Saved output.png");

        // Check ANSI support
        const hasANSI = detectANSISupport();

        if (!hasANSI) {
            // No ANSI support: static output only
            const asciiArt = renderASCIIArt(pixels, width, height);
            console.log("\nASCII Art Render:\n");
            console.log(asciiArt);
        } else {
            // ANSI support: run animation
            await runTerminalAnimation(width, height, 20000);
        }
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
            #controls {
                display: flex;
                gap: 15px;
                align-items: center;
            }
            button {
                padding: 8px 16px;
                font-size: 14px;
                background: #444;
                color: #fff;
                border: 1px solid #666;
                border-radius: 4px;
                cursor: pointer;
                font-family: monospace;
            }
            button:hover {
                background: #555;
            }
            button:active {
                background: #333;
            }
            #fps {
                font-size: 14px;
                color: #aaa;
                min-width: 80px;
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

        // Create controls container
        const controls = document.createElement('div');
        controls.id = 'controls';

        // Create play/pause button
        const button = document.createElement('button');
        button.textContent = '▶ Play';
        button.onclick = () => {
            animationState.running = !animationState.running;
            if (animationState.running) {
                button.textContent = '⏸ Pause';
                animationState.frameCount = 0;
                animationState.lastFpsTime = Date.now();
                animationState.startTime = Date.now();
                requestAnimationFrame(animate);
            } else {
                button.textContent = '▶ Play';
            }
        };
        controls.appendChild(button);

        // Create FPS display
        const fpsDisplay = document.createElement('div');
        fpsDisplay.id = 'fps';
        fpsDisplay.textContent = 'FPS: 0';
        controls.appendChild(fpsDisplay);

        document.body.appendChild(controls);

        // Create canvas
        const canvas = document.createElement('canvas');
        canvas.width = width;
        canvas.height = height;
        document.body.appendChild(canvas);

        const ctx = canvas.getContext('2d');

        // Store references in animationState
        animationState.button = button;
        animationState.canvas = canvas;
        animationState.ctx = ctx;
        animationState.width = width;
        animationState.height = height;
        animationState.fpsElement = fpsDisplay;

        // Display initial frame
        await displayFrame(pixels, width, height);

        console.log("Rendered cube to canvas");
    }
}

main();
