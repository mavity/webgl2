#version 300 es

// Simple vertex shader for Phase 0 testing

in vec3 position;

void main() {
    gl_Position = vec4(position, 1.0);
}
