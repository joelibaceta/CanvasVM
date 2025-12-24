//! # canvas_codegen
//!
//! WASM code generator for Piet bytecode.
//!
//! This crate compiles the intermediate bytecode representation into native
//! WebAssembly binary format. The generated WASM can be executed directly
//! by any compliant runtime (V8, Wasmtime, Wasmer, etc.) without interpretation.
//!
//! ## Architecture
//!
//! ```text
//! ┌──────────┐    ┌───────────┐    ┌─────────────┐    ┌─────────────┐
//! │  Piet    │ →  │ Bytecode  │ →  │ WASM Binary │ →  │ Native      │
//! │  Image   │    │ (IR)      │    │ (.wasm)     │    │ Execution   │
//! └──────────┘    └───────────┘    └─────────────┘    └─────────────┘
//!                 canvas_vm        canvas_codegen     V8/Wasmtime
//! ```
//!
//! ## Memory Layout
//!
//! The generated WASM module uses linear memory as a stack:
//!
//! ```text
//! Memory (64KB page):
//! ┌────────────────────────────────────────────────────────┐
//! │ 0x0000: Stack Pointer (i32)                            │
//! │ 0x0004: Stack Data [i32, i32, i32, ...]                │
//! │         ↑ grows upward                                 │
//! └────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Imports
//!
//! The generated module imports I/O functions from the host:
//!
//! - `env.read_char() -> i32` - Read a character (Unicode codepoint)
//! - `env.read_number() -> i32` - Read an integer
//! - `env.write_char(i32)` - Write a character
//! - `env.write_number(i32)` - Write an integer

mod wasm;

pub use wasm::{WasmCodegen, CodegenError, CodegenOptions};

use canvas_vm::Program;

/// Compile a Piet program to WebAssembly binary format.
///
/// This is the main entry point for code generation.
///
/// # Example
///
/// ```ignore
/// use canvas_vm::Program;
/// use canvas_codegen::compile_to_wasm;
///
/// let program: Program = /* ... */;
/// let wasm_bytes = compile_to_wasm(&program)?;
///
/// // wasm_bytes can now be instantiated in any WASM runtime
/// ```
pub fn compile_to_wasm(program: &Program) -> Result<Vec<u8>, CodegenError> {
    let codegen = WasmCodegen::new();
    codegen.generate(program)
}

/// Compile with custom options.
pub fn compile_to_wasm_with_options(
    program: &Program,
    options: CodegenOptions,
) -> Result<Vec<u8>, CodegenError> {
    let codegen = WasmCodegen::with_options(options);
    codegen.generate(program)
}

#[cfg(test)]
mod tests {
    use super::*;
    use canvas_vm::{Instruction, Program, ProgramMetadata, RichInstruction};

    fn make_program(instructions: Vec<Instruction>) -> Program {
        let rich = instructions.iter()
            .map(|op| RichInstruction::simple(op.clone()))
            .collect();
        Program {
            metadata: ProgramMetadata::default(),
            rich_instructions: rich,
            instructions,
            position_map: vec![],
            next_position: vec![],
            width: 0,
            height: 0,
        }
    }

    #[test]
    fn test_empty_program() {
        let program = make_program(vec![Instruction::Halt]);
        let wasm = compile_to_wasm(&program).unwrap();
        
        // WASM magic number: \0asm
        assert_eq!(&wasm[0..4], b"\0asm");
        // WASM version 1
        assert_eq!(&wasm[4..8], &[1, 0, 0, 0]);
        
        // Validate with wasmparser
        wasmparser::validate(&wasm).expect("Generated WASM should be valid");
    }

    #[test]
    fn test_simple_arithmetic() {
        let program = make_program(vec![
            Instruction::Push(3),
            Instruction::Push(2),
            Instruction::Add,
            Instruction::OutNumber,
            Instruction::Halt,
        ]);
        let wasm = compile_to_wasm(&program).unwrap();
        
        // Should produce valid WASM
        assert!(wasm.len() > 8);
        wasmparser::validate(&wasm).expect("Arithmetic WASM should be valid");
    }

    #[test]
    fn test_stack_operations() {
        let program = make_program(vec![
            Instruction::Push(42),
            Instruction::Duplicate,
            Instruction::Pop,
            Instruction::OutNumber,
            Instruction::Halt,
        ]);
        let wasm = compile_to_wasm(&program).unwrap();
        assert!(wasm.len() > 8);
        wasmparser::validate(&wasm).expect("Stack ops WASM should be valid");
    }

    #[test]
    fn test_all_instructions() {
        // Test that all instruction types generate valid WASM
        let program = make_program(vec![
            Instruction::Push(10),
            Instruction::Push(5),
            Instruction::Add,
            Instruction::Push(3),
            Instruction::Subtract,
            Instruction::Push(2),
            Instruction::Multiply,
            Instruction::Push(4),
            Instruction::Divide,
            Instruction::Push(3),
            Instruction::Mod,
            Instruction::Not,
            Instruction::Push(5),
            Instruction::Greater,
            Instruction::Duplicate,
            Instruction::Pop,
            Instruction::Push(2),
            Instruction::Push(1),
            Instruction::Roll,
            Instruction::Nop,
            Instruction::OutNumber,
            Instruction::Halt,
        ]);
        let wasm = compile_to_wasm(&program).unwrap();
        wasmparser::validate(&wasm).expect("All instructions WASM should be valid");
    }

    #[test]
    fn test_hello_world_pattern() {
        // Simulates output of "Hi" (H=72, i=105)
        let program = make_program(vec![
            Instruction::Push(72),   // 'H'
            Instruction::OutChar,
            Instruction::Push(105),  // 'i'
            Instruction::OutChar,
            Instruction::Halt,
        ]);
        let wasm = compile_to_wasm(&program).unwrap();
        wasmparser::validate(&wasm).expect("Hello world WASM should be valid");
        
        // Check size is reasonable
        println!("WASM size for 'Hi': {} bytes", wasm.len());
        assert!(wasm.len() < 1000, "WASM should be compact");
    }
}
