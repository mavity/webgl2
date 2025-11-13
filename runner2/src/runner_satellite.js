/*
Satellite Node.js script used to run a wasm test binary and collect V8 coverage.
It expects CLI args: --target-wasm <path> --out <outdir> --harness <harness>
The script uses the inspector API to collect precise coverage around the test run
and writes a coverage file to <outdir>/v8-coverage.json.
*/
(async () => {
  const fs = require('fs');
  const path = require('path');
  const inspector = require('inspector');

  function parseArgs(argv) {
    const opts = { target_wasm: null, out: null, harness: null };
    for (let i = 0; i < argv.length; i++) {
      const a = argv[i];
      if (a === '--target-wasm' && i + 1 < argv.length) opts.target_wasm = argv[++i];
      else if (a === '--out' && i + 1 < argv.length) opts.out = argv[++i];
      else if (a === '--harness' && i + 1 < argv.length) opts.harness = argv[++i];
    }
    return opts;
  }

  function post(session, method, params) {
    return new Promise((resolve, reject) => {
      session.post(method, params || {}, (err, res) => {
        if (err) reject(err);
        else resolve(res);
      });
    });
  }

  const opts = parseArgs(process.argv.slice(2));
  if (!opts.target_wasm) {
    console.error('missing --target-wasm');
    process.exit(2);
  }
  const outDir = opts.out || 'coverage';

  try {
    const session = new inspector.Session();
    session.connect();
    await post(session, 'Profiler.enable');
    await post(session, 'Profiler.startPreciseCoverage', { callCount: true, detailed: true });

    // instantiate the wasm module and run tests
    const wasmPath = path.resolve(opts.target_wasm);
    const bytes = fs.readFileSync(wasmPath);
    let exitCode = 0;
    try {
      const module = await WebAssembly.compile(bytes);
      const imports = {};
      // Provide a minimal env if needed
      imports.env = imports.env || {};
      // attach inspector-specific functions? keep imports minimal so tests can run
      const { instance } = await WebAssembly.instantiate(module, imports);
      // Run canonical test harness entrypoints
      if (typeof instance.exports.__run_tests === 'function') {
        instance.exports.__run_tests();
      } else if (typeof instance.exports._start === 'function') {
        instance.exports._start();
      } else if (typeof instance.exports.main === 'function') {
        // main may take args; assume no args for test cases
        instance.exports.main();
      }
    } catch (e) {
      // If wasm cannot be instantiated for tests, log and set exit nonzero
      console.error('wasm test run failed: ', e && e.stack ? e.stack : e);
      exitCode = 1;
    }

    // Take coverage
    const covRes = await post(session, 'Profiler.takePreciseCoverage');
    await post(session, 'Profiler.stopPreciseCoverage');
    await post(session, 'Profiler.disable');
    session.disconnect();

    // Ensure outDir exists, write coverage
    fs.mkdirSync(outDir, { recursive: true });
    const covPath = path.join(outDir, 'v8-coverage.json');
    fs.writeFileSync(covPath, JSON.stringify(covRes, null, 2), 'utf8');

    // Provide a compact JSON summary on stdout for the caller
    console.log(JSON.stringify({ exitCode, coverageFile: covPath }));
    process.exit(exitCode);
  } catch (err) {
    console.error('satellite error', err && err.stack ? err.stack : err);
    process.exit(2);
  }
})();
