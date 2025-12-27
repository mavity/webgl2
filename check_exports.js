
import fs from 'fs';
const wasmBuffer = fs.readFileSync('webgl2.wasm');
const imports = {
    env: new Proxy({}, {
        get: (target, prop) => {
            return () => {};
        }
    })
};
WebAssembly.instantiate(wasmBuffer, imports).then(result => {
  const exports = result.instance.exports;
  // List all exports starting with wasm_ctx_get_vertex_attrib
  for (const key in exports) {
    if (key.startsWith('wasm_ctx_get_vertex_attrib')) {
      console.log(key);
    }
  }
}).catch(e => console.error(e));
