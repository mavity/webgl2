
/**
 * GLSL snippet to pack a float into RGBA8.
 * Replaces 'val' with the expression to pack.
 */
export const PACK_FLOAT_GLSL = (val) => `
    {
        uint u = floatBitsToUint(${val});
        outColor = vec4(
            float(u & 0xFFu) / 255.0,
            float((u >> 8u) & 0xFFu) / 255.0,
            float((u >> 16u) & 0xFFu) / 255.0,
            float((u >> 24u) & 0xFFu) / 255.0
        );
    }
`;

/**
 * Unpacks a Float32 from a 4-byte Uint8Array.
 */
export function unpackFloat(rgba8) {
    const u32 = (rgba8[0]) | (rgba8[1] << 8) | (rgba8[2] << 16) | (rgba8[3] << 24);
    const view = new DataView(new ArrayBuffer(4));
    view.setUint32(0, u32, true);
    return view.getFloat32(0, true);
}

/**
 * Canonicalizes a float for comparison.
 */
export function canonicalize(val) {
    if (isNaN(val)) return "NaN";
    if (!isFinite(val)) return val > 0 ? "Infinity" : "-Infinity";
    // Standardize to Float32 precision
    const f32 = Math.fround(val);
    // Treat very small values as zero to avoid signed zero/precision issues
    if (Math.abs(f32) < 1e-6) return "0.000000";
    return f32.toFixed(6);
}
