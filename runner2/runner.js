#!/usr/bin/env node
// @ts-check

(async () => {
  const fs = require('fs');

  const wasmBuf = fs.readFileSync(__filename.replace(/\.js$/, '.wasm'));

  const wasm = await WebAssembly.instantiate(wasmBuf, { env: { eval: evalBin } });

  const encoder = new TextEncoder();
  const decoder = new TextDecoder('utf-8');

  const argsBytes = encoder.encode(process.argv.join('\0') + '\0');
  const prevPages = wasm.instance.exports.memory.grow(Math.ceil(argsBytes.length / 0x10000) || 1);

  let memArr = new Uint8Array(wasm.instance.exports.memory.buffer);
  const argvPtr = prevPages * 0x10000;
  memArr.set(argsBytes, argvPtr);

  const exitCode = wasm.instance.exports.run(argvPtr);
  process.exit(exitCode);

  /** @param {number} jsPtr @param {number} scratchPtr */
  function evalBin(jsPtr, scratchPtr) {
    memArr = new Uint8Array(wasm.instance.exports.memory.buffer);

    const js = decoder.decode(
      memArr.subarray(jsPtr, memArr.indexOf(0, jsPtr)));

    try {
      let result = undefined;
      try { result = eval(js); } catch (e) { writeOut({ error: String(e) }); return; }
      try { writeOut({ result }); } catch (e) { writeOut({ error: 'Result not serializable' }); }
      return;
    } catch (e) {
      writeOut({ error: e && e.message ? e.message : String(e) });
    }

    /** @param {any} v */
    function writeOut(v) {
      const bytes = encoder.encode(JSON.stringify(v));
      memArr.set(bytes, scratchPtr);
      memArr[scratchPtr + bytes.length] = 0;
    }
  }
})();
