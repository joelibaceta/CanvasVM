// piet - Execute Piet programs
use anyhow::{Context, Result};
use canvas_vm::{BytecodeVm, Grid, Program};
use std::env;
use std::path::PathBuf;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: piet <program.png> [options]");
        eprintln!("       piet <program.cvm>           # Execute pre-compiled bytecode");
        eprintln!();
        eprintln!("Options:");
        eprintln!("  -i, --input <text>              Input text");
        eprintln!("  -n, --numbers <nums>            Input numbers (comma-separated)");
        eprintln!("  -c, --codel-size <size>         Codel size (auto-detect if not specified)");
        eprintln!("  -v, --verbose                   Verbose output");
        eprintln!("  --jit                           Use JIT compilation (faster)");
        std::process::exit(1);
    }

    let file = PathBuf::from(&args[1]);

    // Check if it's a .png/.bmp or pre-compiled bytecode
    let ext = file.extension().and_then(|s| s.to_str()).unwrap_or("");

    match ext {
        "png" | "bmp" => {
            // Load and execute image
            let img = image::open(&file)
                .with_context(|| format!("Failed to load image: {:?}", file))?
                .to_rgba8();

            let (width, height) = img.dimensions();
            let rgba = img.into_raw();

            let grid =
                Grid::from_rgba_with_codel_size(width as usize, height as usize, &rgba, None)?;
            let mut vm = BytecodeVm::from_grid(grid)?;

            // Simple execution
            vm.play(1_000_000)?;
            print!("{}", vm.ink_string());

            Ok(())
        }
        "cvm" => {
            // Load pre-compiled bytecode
            let program = Program::load_from_file(&file)
                .map_err(|e| anyhow::anyhow!("Failed to load bytecode: {}", e))?;

            let mut vm = BytecodeVm::from_program(program)?;
            vm.play(1_000_000)?;
            print!("{}", vm.ink_string());

            Ok(())
        }
        _ => {
            eprintln!("Error: Unsupported file format: .{}", ext);
            eprintln!("Supported: .png, .bmp, .cvm");
            std::process::exit(1);
        }
    }
}
