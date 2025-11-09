# Test Fixtures

This directory contains GLSL shader files for testing the WebGL2 compiler and emulator.

## Files

- `simple.vert` - Minimal vertex shader (passthrough position)
- `simple.frag` - Minimal fragment shader (solid red color)

## Usage

These shaders are used for Phase 0 validation testing:

```bash
# Compile vertex shader
cargo run --bin webgl2 -- compile tests/fixtures/simple.vert --debug

# Validate fragment shader
cargo run --bin webgl2 -- validate tests/fixtures/simple.frag
```
