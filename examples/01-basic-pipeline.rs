//! Example: Complete compilation pipeline
//! 
//! This example demonstrates the full flow from GLSL source to WASM execution

use webgl2::naga_wasm_backend::{WasmBackend, WasmBackendConfig};
use webgl2::wasm_gl_emu::ShaderRuntime;

fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("WebGL2 Emulator - Complete Pipeline Example\n");

    // Step 1: Define a simple GLSL vertex shader
    let glsl_source = r#"
        #version 300 es
        in vec3 position;
        void main() {
            gl_Position = vec4(position, 1.0);
        }
    "#;

    println!("Step 1: GLSL Source");
    println!("{}\n", glsl_source);

    // Step 2: Parse GLSL to Naga IR
    println!("Step 2: Parsing GLSL with Naga...");
    let options = naga::front::glsl::Options::from(naga::ShaderStage::Vertex);
    let module = naga::front::glsl::Frontend::default()
        .parse(&options, glsl_source)
        .map_err(|e| anyhow::anyhow!("GLSL parse error: {:?}", e))?;
    
    println!("✓ Parsed successfully");
    println!("  Entry points: {}", module.entry_points.len());
    println!();

    // Step 3: Validate Naga IR
    println!("Step 3: Validating Naga IR...");
    let mut validator = naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::all(),
    );
    let info = validator.validate(&module)?;
    println!("✓ Validation passed\n");

    // Step 4: Compile to WASM
    println!("Step 4: Compiling to WASM...");
    let config = WasmBackendConfig {
        debug_info: true,
        optimize: false,
        features: Default::default(),
    };

    let backend = WasmBackend::new(config);
    let wasm_module = backend.compile(&module, &info, glsl_source)?;

    println!("✓ Compiled successfully");
    println!("  WASM size: {} bytes", wasm_module.wasm_bytes.len());
    println!("  Entry points: {:?}", wasm_module.entry_points.keys());
    println!("  Memory layout: {:?}", wasm_module.memory_layout);
    println!();

    // Step 5: Load into emulator runtime
    println!("Step 5: Loading into emulator runtime...");
    let mut runtime = ShaderRuntime::new(&wasm_module)?;
    println!("✓ Runtime initialized\n");

    // Step 6: Execute shader
    println!("Step 6: Executing vertex shader...");
    let vertex_data = vec![0.0, 1.0, 0.0]; // Single vertex at (0, 1, 0)
    
    let entry_point = module.entry_points[0].name.clone();
    let result = runtime.run_vertex_shader(&entry_point, &vertex_data)?;

    println!("✓ Execution completed");
    println!("  Output position: {:?}", result);
    println!("  (Note: Phase 0 returns hardcoded values)\n");

    println!("======================");
    println!("Pipeline test complete!");
    println!("======================");

    Ok(())
}
