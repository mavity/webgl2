// Simple Node.js runner to load a compiled shader WASM and call an exported function
// Usage: node node_runner.js path/to/shader.wasm entryName

const fs = require('fs');

if (process.argv.length < 3) {
  console.error('Usage: node node_runner.js path/to/shader.wasm [entryName]');
  process.exit(1);
}

const wasmPath = process.argv[2];
const entryName = process.argv[3] || 'main';

(async () => {
  try {
    const bytes = fs.readFileSync(wasmPath);
    const module = await WebAssembly.compile(bytes);
    const imports = {};
    const instance = await WebAssembly.instantiate(module, imports);

    console.log('WASM instantiated. Exports:', Object.keys(instance.exports));

    if (instance.exports[entryName]) {
      // Attempt to call a function with 3 i32 params returning 4 f32s is nontrivial in Node JS
      // We'll try calling it with dummy integers; the JS runtime will coerce values
      try {
        const result = instance.exports[entryName](0, 0, 0);
        console.log('Called', entryName, '->', result);
      } catch (e) {
        console.warn('Failed to call exported function directly:', e.message);
        console.warn('You may need a JS glue layer or use wasm-bindgen for complex signatures.');
      }
    } else {
      console.warn('Entry not found in exports:', entryName);
    }
  } catch (e) {
    console.error('Error:', e);
    process.exit(2);
  }
})();
