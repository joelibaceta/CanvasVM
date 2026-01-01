mod bytecode;
mod compiler;
mod debugger;
mod error;
mod exits;
mod grid;
mod io;
mod ops;
mod vm;

pub use bytecode::{Instruction, InstructionDebugInfo, Program, ProgramMetadata, RichInstruction};
pub use compiler::{CompileMode, Compiler};
pub use debugger::{
    Debugger, DebuggerState, ExecutionMode, ExecutionStep, ExecutionTrace, InputRequest,
};
pub use error::VmError;
pub use exits::{CodelChooser, Direction, Position};
pub use grid::{BlockId, BlockInfo, Grid};
pub use io::{BufferedInput, BufferedOutput, Input, InputSource, Output, OutputSink};
pub use ops::PietColor;
pub use vm::BytecodeVm;

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    // Helper para cargar imágenes PNG usando la crate image
    fn load_image_to_grid(path: &str) -> Result<Grid, Box<dyn std::error::Error>> {
        let img = image::open(path)?;
        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();

        // Auto-detect codel_size
        Ok(Grid::from_rgba(
            width as usize,
            height as usize,
            rgba.as_raw(),
        )?)
    }

    #[test]
    fn test_hello_world() {
        let grid = load_image_to_grid("../../tools/fixtures/samples/HelloWorld.png")
            .expect("Failed to load HelloWorld.png");

        let mut vm = BytecodeVm::from_grid(grid).expect("VM creation failed");

        // Run with limit
        let max_steps = 10000;
        let mut steps = 0;

        while vm.stroke().is_ok() {
            steps += 1;
            if steps >= max_steps {
                break;
            }
        }

        let output = vm.ink_string();
        println!("HelloWorld output: '{}'", output);

        // Verificar que imprime "Hello World!" o algo similar
        assert!(!output.is_empty(), "Should produce output");
        assert!(
            output.contains("Hello") || output.contains("World"),
            "Output should contain 'Hello' or 'World', got: '{}'",
            output
        );
    }

    #[test]
    fn test_piet_program() {
        let img =
            image::open("../../tools/fixtures/samples/Piet.png").expect("Failed to load image");
        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();

        // Probar auto-detección
        let grid = Grid::from_rgba(width as usize, height as usize, rgba.as_raw())
            .expect("Failed to create grid");

        println!("\n=== GRID INFO (Auto-detected) ===");
        println!("Grid dimensions: {}x{}", grid.width(), grid.height());
        println!("Codel size: {}", grid.codel_size());

        // Inspeccionar los primeros codels
        println!("\nFirst 10 codels in row 0:");
        for x in 0..10 {
            let pos = Position::new(x, 0);
            let color = grid.get(pos);
            println!("  ({}, 0): {:?}", x, color);
        }

        // Primero, inspeccionar el bytecode generado
        let vm = BytecodeVm::from_grid(grid.clone()).expect("VM creation failed");
        println!("\n=== BYTECODE ANALYSIS ===");
        println!("First 10 instructions:");
        for (i, instr) in vm.instructions().iter().take(10).enumerate() {
            println!("[{}] {:?}", i, instr);
        }

        // Ahora ejecutar paso a paso
        let mut vm = BytecodeVm::from_grid(grid).expect("VM creation failed");

        println!("\n=== EXECUTION TRACE ===");
        for i in 0..100 {
            let stack_top = vm.peek();
            let output_before = vm.ink_string();

            match vm.stroke() {
                Ok(_) => {}
                Err(_) => break,
            }

            let output_after = vm.ink_string();

            // Detener si hubo nuevo output
            if output_after.len() > output_before.len() {
                let new_char = output_after.chars().last().unwrap();
                println!("\n[Step {}] OUTPUT PRODUCED!", i);
                println!("  Stack top before stroke: {:?}", stack_top);
                println!(
                    "  Character: '{}' (U+{:04X}, decimal: {})",
                    new_char, new_char as u32, new_char as u32
                );
                println!("  Full output so far: '{}'", output_after);
                break;
            }
        }

        let output = vm.ink_string();
        println!("\nFinal output: '{}'", output);
        for (i, ch) in output.chars().enumerate() {
            println!(
                "  char[{}]: '{}' (U+{:04X}, decimal: {})",
                i, ch, ch as u32, ch as u32
            );
        }

        // Verificar que al menos ejecuta y produce algún output
        let steps = vm.get_steps();
        assert!(steps > 0, "Should execute at least one step");
        assert!(!output.is_empty(), "Should produce some output");
    }

    #[test]
    fn test_pi_program() {
        let grid = load_image_to_grid("../../tools/fixtures/samples/PI.png")
            .expect("Failed to load PI.png");

        println!("\n=== PI.png ===");
        println!(
            "Grid: {}x{}, codel_size: {}",
            grid.width(),
            grid.height(),
            grid.codel_size()
        );

        let mut vm = BytecodeVm::from_grid(grid).expect("VM creation failed");

        // Run with limit
        let _ = vm.play(10000);

        let output = vm.ink_string();
        println!("Output: '{}'", output);

        assert!(!output.is_empty(), "Should produce output");
    }

    #[test]
    fn test_az_program() {
        let img = image::open("../../tools/fixtures/samples/AZ.png").expect("Failed to load image");
        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();

        // Auto-detect codel_size using new algorithm
        let grid = Grid::from_rgba(width as usize, height as usize, rgba.as_raw())
            .expect("Failed to create grid");

        println!("\n=== AZ.png ===");
        println!(
            "Grid: {}x{}, codel_size: {}",
            grid.width(),
            grid.height(),
            grid.codel_size()
        );

        let mut vm = BytecodeVm::from_grid(grid).expect("VM creation failed");

        println!("First 10 instructions:");
        for (i, instr) in vm.instructions().iter().take(10).enumerate() {
            println!("[{}] {:?}", i, instr);
        }

        // Run with limit
        let _ = vm.play(10000);

        let output = vm.ink_string();
        println!("Output: '{}'", output);

        assert_eq!(
            output, "abcdefghijklmnopqrstuvwxyz",
            "Should print lowercase alphabet"
        );
    }

    #[test]
    fn test_hello_world3() {
        let grid = load_image_to_grid("../../tools/fixtures/samples/HelloWorld3.png")
            .expect("Failed to load HelloWorld3.png");

        println!("\n=== HelloWorld3.png ===");
        println!(
            "Grid: {}x{}, codel_size: {}",
            grid.width(),
            grid.height(),
            grid.codel_size()
        );

        let mut vm = BytecodeVm::from_grid(grid).expect("VM creation failed");

        // Run with limit
        let _ = vm.play(10000);

        let output = vm.ink_string();
        println!("Output: '{}'", output);

        assert_eq!(output.trim(), "Hello, world!", "Should print Hello, world!");
    }

    #[test]
    fn test_sum() {
        let grid = load_image_to_grid("../../tools/fixtures/samples/Sum.png")
            .expect("Failed to load Sum.png");

        println!("\n=== Sum.png ===");
        println!(
            "Grid: {}x{}, codel_size: {}",
            grid.width(),
            grid.height(),
            grid.codel_size()
        );

        let mut vm = BytecodeVm::from_grid(grid).expect("VM creation failed");

        // Mock stdin with two numbers: 5 and 7
        vm.load_input_number_vec(&[5, 7]);

        // Run with limit
        let _ = vm.play(10000);

        let output = vm.ink_string();
        println!("Output: '{}'", output);

        // Expected: prompts "n? n? " and outputs "12" (5+7)
        assert!(output.contains("12"), "Should output sum of 5+7=12");
    }

    #[test]
    fn test_hello_world2() {
        let grid = load_image_to_grid("../../tools/fixtures/samples/HelloWorld2.png")
            .expect("Failed to load HelloWorld2.png");

        println!("\n=== HelloWorld2.png ===");
        println!(
            "Grid: {}x{}, codel_size: {}",
            grid.width(),
            grid.height(),
            grid.codel_size()
        );

        let mut vm = BytecodeVm::from_grid(grid).expect("VM creation failed");

        // Mock stdin with input characters
        vm.load_input_text("ab");

        // Run with limit
        let _ = vm.play(10000);

        let output = vm.ink_string();
        println!("Output: '{}'", output);

        // Expected: "Hello world" (with input would echo chars)
        assert!(
            output.contains("Hello world"),
            "Should contain 'Hello world'"
        );
    }

    #[test]
    fn test_echo4() {
        // Load the BMP
        let data =
            fs::read("../../tools/fixtures/samples/echo4_terminating.bmp").expect("File not found");

        // Parse BMP header
        let width = u32::from_le_bytes([data[18], data[19], data[20], data[21]]) as usize;
        let height = u32::from_le_bytes([data[22], data[23], data[24], data[25]]) as usize;
        let offset = u32::from_le_bytes([data[10], data[11], data[12], data[13]]) as usize;

        println!("Image: {}x{}", width, height);

        // Convert BMP to Grid
        let stride = (width * 3).div_ceil(4) * 4;
        let mut pixels = vec![];

        for y in 0..height {
            for x in 0..width {
                // BMP is bottom-up
                let bmp_y = height - 1 - y;
                let pos = offset + bmp_y * stride + x * 3;
                let b = data[pos];
                let g = data[pos + 1];
                let r = data[pos + 2];
                pixels.push(r);
                pixels.push(g);
                pixels.push(b);
                pixels.push(255); // alpha
            }
        }

        let grid = Grid::from_rgba(width, height, &pixels).expect("Grid creation failed");

        println!("Grid dimensions: {}x{}", grid.width(), grid.height());
        println!(
            "Starting position color: {:?}",
            grid.get(Position::new(0, 0))
        );

        // Use the same compilation path as the web editor
        let compiler = Compiler::with_codel_size(grid.clone(), 1, width, height);
        let program = compiler.compile().expect("Compilation failed");

        println!("Compiled {} instructions", program.instructions.len());
        for (i, instr) in program.instructions.iter().enumerate().take(15) {
            println!("  [{}] {:?}", i, instr);
        }

        let mut vm = BytecodeVm::new(program, grid);

        // Provide input: "HOLA"
        vm.load_input_text("HOLA");

        // Run with limit
        let max_steps = 10000;
        let mut steps = 0;

        loop {
            match vm.stroke() {
                Ok(_) => {
                    steps += 1;
                    if steps >= max_steps {
                        println!("Max steps reached");
                        break;
                    }
                }
                Err(e) => {
                    println!("Stopped after {} steps: {:?}", steps, e);
                    break;
                }
            }
        }

        let output = vm.ink_string();
        println!("Output: '{}'", output);

        // El programa echo4_terminating simplemente termina sin leer/imprimir
        // Solo verificamos que termine correctamente
        assert!(steps > 0, "Should execute at least one step");
        assert!(steps < max_steps, "Should terminate before max steps");
    }
}
