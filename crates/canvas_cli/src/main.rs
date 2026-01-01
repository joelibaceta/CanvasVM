use anyhow::{Context, Result};
use canvas_codegen::compile_to_wasm;
use canvas_vm::{BytecodeVm, Compiler, Debugger, Grid};
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use wasmtime::*;

#[derive(Parser)]
#[command(name = "canvas-vm")]
#[command(about = "Canvas VM - A high-performance Piet language runtime", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run a Piet program
    Run {
        /// Path to the Piet image file (PNG or BMP)
        #[arg(value_name = "FILE")]
        file: PathBuf,

        /// Codel size (auto-detect if not specified)
        #[arg(short, long)]
        codel_size: Option<usize>,

        /// Input text for the program
        #[arg(short, long)]
        input: Option<String>,

        /// Input numbers (comma-separated)
        #[arg(short = 'n', long)]
        numbers: Option<String>,

        /// Maximum steps before timeout (default: 1000000)
        #[arg(short = 'l', long, default_value = "1000000")]
        limit: usize,

        /// Verbose output
        #[arg(short, long)]
        verbose: bool,

        /// Use native JIT compilation (faster, via Wasmtime)
        #[arg(long)]
        jit: bool,
    },

    /// Debug a Piet program step by step
    Debug {
        /// Path to the Piet image file
        #[arg(value_name = "FILE")]
        file: PathBuf,

        /// Codel size (auto-detect if not specified)
        #[arg(short, long)]
        codel_size: Option<usize>,

        /// Breakpoint at instruction index
        #[arg(short, long)]
        breakpoint: Option<usize>,
    },

    /// Compile a Piet program to bytecode
    Compile {
        /// Path to the Piet image file
        #[arg(value_name = "FILE")]
        file: PathBuf,

        /// Codel size (auto-detect if not specified)
        #[arg(short, long)]
        codel_size: Option<usize>,

        /// Show debug information
        #[arg(short, long)]
        verbose: bool,
    },

    /// Analyze a Piet image
    Analyze {
        /// Path to the Piet image file
        #[arg(value_name = "FILE")]
        file: PathBuf,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run {
            file,
            codel_size,
            input,
            numbers,
            limit,
            verbose,
            jit,
        } => {
            if jit {
                run_program_jit(file, codel_size, input, numbers, limit, verbose)
            } else {
                run_program(file, codel_size, input, numbers, limit, verbose)
            }
        }

        Commands::Debug {
            file,
            codel_size,
            breakpoint,
        } => debug_program(file, codel_size, breakpoint),

        Commands::Compile {
            file,
            codel_size,
            verbose,
        } => compile_program(file, codel_size, verbose),

        Commands::Analyze { file } => analyze_image(file),
    }
}

fn load_image(file: PathBuf) -> Result<(image::RgbaImage, usize, usize)> {
    let img = image::open(&file)
        .with_context(|| format!("Failed to load image: {:?}", file))?
        .to_rgba8();

    let (width, height) = img.dimensions();
    Ok((img, width as usize, height as usize))
}

fn run_program_jit(
    file: PathBuf,
    codel_size_opt: Option<usize>,
    _input: Option<String>,
    _numbers: Option<String>,
    _limit: usize,
    verbose: bool,
) -> Result<()> {
    let (img, width, height) = load_image(file)?;
    let rgba = img.into_raw();

    let grid = Grid::from_rgba_with_codel_size(width, height, &rgba, codel_size_opt)?;

    if verbose {
        eprintln!(
            "Grid: {}x{}, codel_size: {}",
            grid.width(),
            grid.height(),
            grid.codel_size()
        );
    }

    // Compile to bytecode
    let compiler = Compiler::new(grid.clone());
    let program = compiler.compile()?;

    if verbose {
        eprintln!("Compiled to {} instructions", program.instructions.len());
    }

    // Generate WASM
    let wasm_bytes = compile_to_wasm(&program)
        .map_err(|e| anyhow::anyhow!("WASM generation failed: {:?}", e))?;

    if verbose {
        eprintln!("Generated {} bytes of WASM", wasm_bytes.len());
        eprintln!("JIT compiling to native x86_64/ARM code...");
    }

    // Create Wasmtime engine (JIT compiler to native)
    let engine = Engine::default();
    let module = Module::from_binary(&engine, &wasm_bytes)?;

    if verbose {
        eprintln!("Native JIT compilation complete!");
    }

    // Create store
    let mut store = Store::new(&engine, ());

    // Create output buffer
    let output = std::sync::Arc::new(std::sync::Mutex::new(String::new()));
    let output_clone = output.clone();

    // Define host functions for I/O
    let read_char = Func::wrap(&mut store, || -> i32 {
        // TODO: implement input
        0
    });

    let read_number = Func::wrap(&mut store, || -> i32 {
        // TODO: implement input
        0
    });

    let write_char = Func::wrap(&mut store, move |val: i32| {
        if let Ok(c) = char::try_from(val as u32) {
            output_clone.lock().unwrap().push(c);
            print!("{}", c);
        }
    });

    let output_clone2 = output.clone();
    let write_number = Func::wrap(&mut store, move |val: i32| {
        output_clone2.lock().unwrap().push_str(&val.to_string());
        print!("{}", val);
    });

    // Link imports
    let imports = [
        read_char.into(),
        read_number.into(),
        write_char.into(),
        write_number.into(),
    ];

    // Instantiate
    let instance = Instance::new(&mut store, &module, &imports)?;

    // Get the main function
    let main = instance.get_typed_func::<(), ()>(&mut store, "main")?;

    // Execute native code!
    main.call(&mut store, ())?;

    if verbose {
        let final_output = output.lock().unwrap();
        eprintln!("\nOutput: {} chars", final_output.len());
    }

    Ok(())
}

fn run_program(
    file: PathBuf,
    codel_size_opt: Option<usize>,
    input: Option<String>,
    numbers: Option<String>,
    limit: usize,
    verbose: bool,
) -> Result<()> {
    let (img, width, height) = load_image(file)?;
    let rgba = img.into_raw();

    // Create grid with auto-detect or specified codel size
    let grid = Grid::from_rgba_with_codel_size(width, height, &rgba, codel_size_opt)?;

    if verbose {
        eprintln!(
            "Grid: {}x{}, codel_size: {}",
            grid.width(),
            grid.height(),
            grid.codel_size()
        );
    }

    // Compile
    let compiler = Compiler::new(grid.clone());
    let program = compiler.compile()?;

    if verbose {
        eprintln!("Compiled to {} instructions", program.instructions.len());
    }

    // Create VM
    let mut vm = BytecodeVm::from_grid(grid)?;

    // Load input
    if let Some(text) = input {
        vm.load_input_text(&text);
    }
    if let Some(nums) = numbers {
        let numbers: Vec<i32> = nums
            .split(',')
            .map(|s| s.trim().parse())
            .collect::<std::result::Result<_, _>>()
            .context("Failed to parse numbers")?;
        vm.load_input_number_vec(&numbers);
    }

    // Execute
    if verbose {
        eprintln!("Executing (limit: {} steps)...", limit);
    }

    match vm.play(limit) {
        Ok(steps) => {
            if verbose {
                eprintln!("Executed {} steps", steps);
            }
            let output = vm.ink_string();
            print!("{}", output);
            Ok(())
        }
        Err(e) => {
            let output = vm.ink_string();
            if !output.is_empty() {
                print!("{}", output);
            }
            anyhow::bail!("{:?}", e)
        }
    }
}

fn debug_program(
    file: PathBuf,
    codel_size_opt: Option<usize>,
    breakpoint: Option<usize>,
) -> Result<()> {
    let (img, width, height) = load_image(file)?;
    let rgba = img.into_raw();

    let grid = Grid::from_rgba_with_codel_size(width, height, &rgba, codel_size_opt)?;

    eprintln!(
        "Grid: {}x{}, codel_size: {}",
        grid.width(),
        grid.height(),
        grid.codel_size()
    );

    let mut debugger = Debugger::new(grid.clone(), grid.codel_size(), width, height)?;

    if let Some(bp) = breakpoint {
        debugger.add_breakpoint(bp);
        eprintln!("Breakpoint set at instruction {}", bp);
    }

    let program = debugger.program();
    eprintln!("Program has {} instructions\n", program.instructions.len());

    // Interactive debug loop
    use std::io::{self, BufRead, Write};
    let stdin = io::stdin();
    let mut input = stdin.lock();

    loop {
        let state = debugger.state();

        if state.halted {
            eprintln!("\n=== HALTED ===");
            eprintln!("Output: {}", state.output_string);
            break;
        }

        // Show current state
        eprintln!("\n=== Step {} ===", state.steps);
        eprintln!("IP: {}", state.ip);
        eprintln!("Position: ({}, {})", state.position.0, state.position.1);
        eprintln!("DP: {:?}, CC: {:?}", state.dp, state.cc);
        eprintln!("Stack: {:?}", state.stack);

        if let Some(ref instr) = state.current_instruction {
            eprintln!("Current: {:?}", instr.op);
        }

        if !state.output_string.is_empty() {
            eprintln!("Output: {}", state.output_string);
        }

        // Prompt
        eprint!("\n[s]tep, [c]ontinue, [q]uit: ");
        io::stderr().flush()?;

        let mut line = String::new();
        input.read_line(&mut line)?;

        match line.trim() {
            "s" | "" => {
                debugger.step()?;
            }
            "c" => {
                debugger.run_limited(100_000)?;
            }
            "q" => {
                break;
            }
            _ => {
                eprintln!("Unknown command");
            }
        }
    }

    Ok(())
}

fn compile_program(file: PathBuf, codel_size_opt: Option<usize>, verbose: bool) -> Result<()> {
    let (img, width, height) = load_image(file)?;
    let rgba = img.into_raw();

    let grid = Grid::from_rgba_with_codel_size(width, height, &rgba, codel_size_opt)?;

    if verbose {
        println!(
            "Grid: {}x{}, codel_size: {}",
            grid.width(),
            grid.height(),
            grid.codel_size()
        );
    }

    let codel_size = grid.codel_size();
    let compiler = Compiler::with_codel_size(grid, codel_size, width, height);
    let program = compiler.compile()?;

    println!(
        "\n=== Bytecode ({} instructions) ===\n",
        program.instructions.len()
    );

    for (i, rich_instr) in program.rich_instructions.iter().enumerate() {
        print!("{:4}: {:?}", i, rich_instr.op);

        if verbose {
            if let Some(ref debug) = rich_instr.debug {
                print!(
                    " ; ({},{}) -> ({},{}), {:?}, {:?}",
                    debug.from_pos.0,
                    debug.from_pos.1,
                    debug.to_pos.0,
                    debug.to_pos.1,
                    debug.dp,
                    debug.cc
                );
            }
        }

        println!();
    }

    Ok(())
}

fn analyze_image(file: PathBuf) -> Result<()> {
    let (img, width, height) = load_image(file)?;
    let rgba = img.into_raw();

    println!("Image: {}x{} pixels", width, height);

    // Auto-detect codel size
    let grid = Grid::from_rgba_with_codel_size(width, height, &rgba, None)?;

    println!("Detected codel size: {}", grid.codel_size());
    println!("Grid dimensions: {}x{} codels", grid.width(), grid.height());

    Ok(())
}
