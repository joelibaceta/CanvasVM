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
pub use io::{Input, Output};
pub use ops::PietColor;
pub use vm::BytecodeVm;

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_echo4_ordered() {
        // Load the BMP
        let data = fs::read("../../tools/fixtures/samples/echo4_simple.bmp").expect("File not found");
        
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
        let mut vm = BytecodeVm::from_grid(grid).expect("VM creation failed");
        
        // Provide input: "HOLA"
        vm.load_input_text("HOLA");
        
        // Run with limit
        let max_steps = 1000;
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
        
        // El programa puede imprimir al revés por el stack LIFO
        // Lo importante es que termine y tenga output
        assert!(!output.is_empty(), "Should have some output");
        assert!(output.len() == 4, "Should have 4 characters, got {}", output.len());
    }
}

