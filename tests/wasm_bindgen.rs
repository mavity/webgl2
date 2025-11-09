// Simple wasm-bindgen-test that only compiles for the wasm32 target.
// This demonstrates the test harness and gives us a minimal test to run.
#![cfg(target_arch = "wasm32")]

use wasm_bindgen_test::*;

// Do not set `run_in_browser` — default test runner runs in Node.js.

#[wasm_bindgen_test]
fn simple_add() {
    // very small, deterministic test useful to verify the test harness
    assert_eq!(1 + 1, 2);
}
