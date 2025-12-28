// Shared matrix math utilities for cube rendering tests

/**
 * Creates a perspective projection matrix
 * @param {number} fovy - Field of view angle in radians (Y axis)
 * @param {number} aspect - Aspect ratio (width/height)
 * @param {number} near - Near clipping plane distance
 * @param {number} far - Far clipping plane distance
 * @returns {number[]} 16-element array representing a 4x4 matrix in column-major order
 */
export function perspective(fovy, aspect, near, far) {
  const f = 1.0 / Math.tan(fovy / 2);
  const nf = 1 / (near - far);
  return [
    f / aspect, 0, 0, 0,
    0, f, 0, 0,
    0, 0, (far + near) * nf, -1,
    0, 0, (2 * far * near) * nf, 0
  ];
}

/**
 * Multiplies two 4x4 matrices
 * @param {number[]|Float32Array} a - First matrix in column-major order
 * @param {number[]|Float32Array} b - Second matrix in column-major order
 * @returns {Float32Array} Result matrix in column-major order
 */
export function multiply(a, b) {
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

/**
 * Creates a translation matrix and multiplies it with the input matrix
 * @param {number[]|Float32Array} m - Input matrix in column-major order
 * @param {number} x - X translation
 * @param {number} y - Y translation
 * @param {number} z - Z translation
 * @returns {Float32Array} Result matrix in column-major order
 */
export function translate(m, x, y, z) {
  const t = [
    1, 0, 0, 0,
    0, 1, 0, 0,
    0, 0, 1, 0,
    x, y, z, 1
  ];
  return multiply(m, t);
}

/**
 * Creates a Y-axis rotation matrix and multiplies it with the input matrix
 * @param {number[]|Float32Array} m - Input matrix in column-major order
 * @param {number} angle - Rotation angle in radians
 * @returns {Float32Array} Result matrix in column-major order
 */
export function rotateY(m, angle) {
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

/**
 * Creates an X-axis rotation matrix and multiplies it with the input matrix
 * @param {number[]|Float32Array} m - Input matrix in column-major order
 * @param {number} angle - Rotation angle in radians
 * @returns {Float32Array} Result matrix in column-major order
 */
export function rotateX(m, angle) {
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

/**
 * 4x4 identity matrix in column-major order
 */
export const identity = [
  1, 0, 0, 0,
  0, 1, 0, 0,
  0, 0, 1, 0,
  0, 0, 0, 1
];
