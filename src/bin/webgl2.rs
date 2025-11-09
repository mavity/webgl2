//! WebGL2 Shader Compiler and Debugger CLI

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use webgl2::{
    glsl_introspection, js_codegen, naga_wasm_backend::WasmBackendConfig,
    naga_wasm_backend::WasmBackend, wasm_gl_emu::ShaderRuntime, naga_wasm_backend::WasmModule,
};

#[derive(Parser)]
#[command(name = "webgl2")]
#[command(about = "WebGL2 shader compiler with debugging support", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Enable verbose logging
    #[arg(short, long, global = true)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Compile GLSL shaders to WASM
    Compile {
        /// Input GLSL file
        #[arg(value_name = "INPUT")]
        input: PathBuf,

        /// Output WASM file
        #[arg(short, long, value_name = "OUTPUT")]
        output: Option<PathBuf>,

        /// Generate DWARF debug information
        #[arg(short, long)]
        debug: bool,
    },

    /// Validate GLSL shaders
    Validate {
        /// Input GLSL file
        #[arg(value_name = "INPUT")]
        input: PathBuf,
    },

    /// Generate TypeScript harness code
    Codegen {
        /// Input GLSL file
        #[arg(value_name = "INPUT")]
        input: PathBuf,

        /// Output TypeScript file
        #[arg(short, long, value_name = "OUTPUT")]
        output: Option<PathBuf>,
    },

    /// Run shader in emulator
    Run {
        /// Compiled WASM file
        #[arg(value_name = "WASM")]
        wasm: PathBuf,

        /// Entry point function name
        #[arg(short, long, default_value = "main")]
        entry: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    let log_level = if cli.verbose {
        tracing::Level::DEBUG
    } else {
        tracing::Level::INFO
    };

    tracing_subscriber::fmt().with_max_level(log_level).init();

    match cli.command {
        Commands::Compile {
            input,
            output,
            debug,
        } => {
            compile_shader(&input, output.as_deref(), debug)?;
        }
        Commands::Validate { input } => {
            validate_shader(&input)?;
        }
        Commands::Codegen { input, output } => {
            generate_code(&input, output.as_deref())?;
        }
        Commands::Run { wasm, entry } => {
            run_shader(&wasm, &entry)?;
        }
    }

    Ok(())
}

fn compile_shader(input: &PathBuf, output: Option<&PathBuf>, debug: bool) -> Result<()> {
    tracing::info!("Compiling shader: {:?}", input);

    // Read input file
    let source = std::fs::read_to_string(input)?;

    // Parse GLSL
    let module = glsl_introspection::parse_glsl(&source)?;

    // Validate
    let mut validator = naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::all(),
    );
    let info = validator.validate(&module)?;

    // Compile to WASM
    let config = WasmBackendConfig {
        debug_info: debug,
        optimize: false,
        features: Default::default(),
    };

    let backend = WasmBackend::new(config);
    let wasm_module = backend.compile(&module, &info, &source)?;

    // Determine output path
    let output_path = output.map(|p| p.to_path_buf()).unwrap_or_else(|| {
        let mut p = input.clone();
        p.set_extension("wasm");
        p
    });

    // Write WASM output
    std::fs::write(&output_path, &wasm_module.wasm_bytes)?;

    tracing::info!(
        "Compiled {} bytes to {:?} ({} entry points)",
        wasm_module.wasm_bytes.len(),
        output_path,
        wasm_module.entry_points.len()
    );

    Ok(())
}

fn validate_shader(input: &PathBuf) -> Result<()> {
    tracing::info!("Validating shader: {:?}", input);

    let source = std::fs::read_to_string(input)?;
    let module = glsl_introspection::parse_glsl(&source)?;

    let mut validator = naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::all(),
    );
    validator.validate(&module)?;

    tracing::info!("✓ Shader is valid");

    Ok(())
}

fn generate_code(input: &PathBuf, output: Option<&PathBuf>) -> Result<()> {
    tracing::info!("Generating TypeScript harness for: {:?}", input);

    let source = std::fs::read_to_string(input)?;
    let manifest = glsl_introspection::introspect_shader(&source)?;
    let typescript = js_codegen::generate_typescript(&manifest)?;

    // Determine output path
    let output_path = output.map(|p| p.to_path_buf()).unwrap_or_else(|| {
        let mut p = input.clone();
        p.set_extension("ts");
        p
    });

    std::fs::write(&output_path, typescript)?;

    tracing::info!("Generated TypeScript harness: {:?}", output_path);

    Ok(())
}

fn run_shader(wasm_path: &PathBuf, entry: &str) -> Result<()> {
    tracing::info!("Running shader: {:?}, entry point: {}", wasm_path, entry);

    // Read WASM file
    let wasm_bytes = std::fs::read(wasm_path)?;

    // Create a WasmModule structure
    let wasm_module = WasmModule {
        wasm_bytes,
        dwarf_bytes: None,
        entry_points: [(entry.to_string(), 0)].into_iter().collect(),
        memory_layout: Default::default(),
    };

    // Create runtime
    let mut runtime = ShaderRuntime::new(&wasm_module)?;

    // Execute shader with dummy data
    let result = runtime.run_vertex_shader(entry, &[])?;

    tracing::info!("Shader output: {:?}", result);

    Ok(())
}
