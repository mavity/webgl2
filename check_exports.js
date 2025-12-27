
import fs from 'fs';
const wasmBuffer = fs.readFileSync('webgl2.debug.wasm');
const imports = {
    env: new Proxy({}, {
        get: (target, prop) => {
            return () => {};
        }
    })
};
WebAssembly.instantiate(wasmBuffer, imports).then(result => {
  const exports = result.instance.exports;
  console.log('wasm_ctx_vertex_attrib_i4i:', typeof exports.wasm_ctx_vertex_attrib_i4i);
  console.log('wasm_ctx_vertex_attrib4f:', typeof exports.wasm_ctx_vertex_attrib4f);
  // List all exports starting with wasm_ctx_vertex_attrib
  for (const key in exports) {
    if (key.startsWith('wasm_ctx_vertex_attrib')) {
      console.log(key);
    }
  }
}).catch(e => console.error(e));
