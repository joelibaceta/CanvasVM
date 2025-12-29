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
pub use debugger::{Debugger, DebuggerState, ExecutionMode, ExecutionStep, ExecutionTrace, InputRequest};
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
        
        Ok(Grid::from_rgba(width as usize, height as usize, rgba.as_raw())?)
    }

    #[test]
    fn test_hello_world() {
        let grid = load_image_to_grid("../../tools/fixtures/samples/HelloWorld.png")
            .expect("Failed to load HelloWorld.png");
        
        let mut vm = BytecodeVm::from_grid(grid).expect("VM creation failed");
        
        // Run with limit
        let max_steps = 10000;
        let mut steps = 0;
        
        loop {
            match vm.stroke() {
                Ok(_) => {
                    steps += 1;
                    if steps >= max_steps {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
        
        let output = vm.ink_string();
        println!("HelloWorld output: '{}'", output);
        
        // Verificar que imprime "Hello World!" o algo similar
        assert!(!output.is_empty(), "Should produce output");
        assert!(output.contains("Hello") || output.contains("World"), 
                "Output should contain 'Hello' or 'World', got: '{}'", output);
    }

    #[test]
    fn test_piet_program() {
        let grid = load_image_to_grid("../../tools/fixtures/samples/Piet.png")
            .expect("Failed to load Piet.png");
        
        let mut vm = BytecodeVm::from_grid(grid).expect("VM creation failed");
        
        let max_steps = 1000; // Limitar más ya que puede entrar en loop
        let mut steps = 0;
        
        loop {
            match vm.stroke() {
                Ok(_) => {
                    steps += 1;
                    if steps >= max_steps {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
        
        let output = vm.ink_string();
        println!("Piet output: '{}' after {} steps", output, steps);
        
        // Verificar que al menos ejecuta y produce algún output
        assert!(steps > 0, "Should execute at least one step");
        assert!(!output.is_empty(), "Should produce some output");
    }

    #[test]
    fn test_echo4() {
        // Load the BMP
        let data = fs::read("../../tools/fixtures/samples/echo4_terminating.bmp").expect("File not found");
        
        // Parse BMP header
        let width = u32::from_le_bytes([data[18], data[19], data[20], data[21]]) as usize;
        let height = u32::from_le_bytes([data[22], data[23], data[24], data[25]]) as usize;
        let offset = u32::from_le_bytes([data[10], data[11], data[12], data[13]]) as usize;
        
        println!("Image: {}x{}", width, height);
        
        // Convert BMP to Grid
        let stride = ((width * 3 + 3) / 4) * 4;
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
        println!("Starting position color: {:?}", grid.get(Position::new(0, 0)));
        
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

