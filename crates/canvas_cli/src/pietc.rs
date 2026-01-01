// pietc - Piet Compiler
use anyhow::{Context, Result};
use canvas_codegen::compile_to_wasm;
use canvas_vm::{Compiler, Grid};
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: pietc <program.png> [options]");
        eprintln!();
        eprintln!("Options:");
        eprintln!("  -o, --output <file>             Output file (default: program.cvm)");
        eprintln!("  -c, --codel-size <size>         Codel size");
        eprintln!("  --wasm                          Compile to WASM instead of bytecode");
        eprintln!("  -v, --verbose                   Verbose output");
        eprintln!();
        eprintln!("Examples:");
        eprintln!("  pietc hello.png                 # Compile to hello.cvm");
        eprintln!("  pietc hello.png -o out.cvm      # Custom output");
        eprintln!("  pietc hello.png --wasm          # Compile to hello.wasm");
        std::process::exit(1);
    }

    let input_file = PathBuf::from(&args[1]);
    let mut output_file = input_file.with_extension("cvm");
    let mut wasm_mode = false;
    let mut verbose = false;
    let mut codel_size = None;

    // Parse arguments
    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "-o" | "--output" => {
                if i + 1 < args.len() {
                    output_file = PathBuf::from(&args[i + 1]);
                    i += 2;
                } else {
                    eprintln!("Error: --output requires a value");
                    std::process::exit(1);
                }
            }
            "-c" | "--codel-size" => {
                if i + 1 < args.len() {
                    codel_size = Some(args[i + 1].parse::<usize>().context("Invalid codel size")?);
                    i += 2;
                } else {
                    eprintln!("Error: --codel-size requires a value");
                    std::process::exit(1);
                }
            }
            "--wasm" => {
                wasm_mode = true;
                output_file = input_file.with_extension("wasm");
                i += 1;
            }
            "-v" | "--verbose" => {
                verbose = true;
                i += 1;
            }
            _ => {
                eprintln!("Unknown option: {}", args[i]);
                std::process::exit(1);
            }
        }
    }

    // Load image
    if verbose {
        eprintln!("📥 Loading {}...", input_file.display());
    }

    let img = image::open(&input_file)
        .with_context(|| format!("Failed to load image: {:?}", input_file))?
        .to_rgba8();

    let (width, height) = img.dimensions();
    let rgba = img.into_raw();

    // Parse grid
    let grid = Grid::from_rgba_with_codel_size(width as usize, height as usize, &rgba, codel_size)?;

    if verbose {
        eprintln!(
            "📐 Grid: {}x{}, codel_size: {}",
            grid.width(),
            grid.height(),
            grid.codel_size()
        );
    }

    // Compile
    let compiler = Compiler::new(grid);
    let program = compiler.compile()?;

    if verbose {
        eprintln!("🔨 Compiled to {} instructions", program.instructions.len());
    }

    if wasm_mode {
        // Generate WASM
        let wasm_bytes = compile_to_wasm(&program)
            .map_err(|e| anyhow::anyhow!("WASM generation failed: {:?}", e))?;

        let mut file = File::create(&output_file)?;
        file.write_all(&wasm_bytes)?;

        if verbose {
            eprintln!(
                "WASM written to {} ({} bytes)",
                output_file.display(),
                wasm_bytes.len()
            );
        } else {
            println!("{}", output_file.display());
        }
    } else {
        // Serialize bytecode to .cvm format
        program
            .save_to_file(&output_file)
            .map_err(|e| anyhow::anyhow!("Failed to save bytecode: {}", e))?;

        if verbose {
            let size = program.serialized_size().unwrap_or(0);
            eprintln!(
                "Bytecode written to {} ({} bytes)",
                output_file.display(),
                size
            );
            eprintln!("   {} instructions compiled", program.instructions.len());
        } else {
            println!("{}", output_file.display());
        }
    }

    Ok(())
}
