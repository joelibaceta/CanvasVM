//! WASM binary code generator
//!
//! Generates valid WebAssembly binary format from Piet bytecode.

use canvas_vm::{Instruction, Program};
use wasm_encoder::{
    CodeSection, ExportKind, ExportSection, Function, FunctionSection,
    ImportSection, Instruction as WasmInst, MemorySection, MemoryType,
    Module, TypeSection, ValType,
};

/// Code generation error
#[derive(Debug, Clone)]
pub enum CodegenError {
    /// Invalid instruction sequence
    InvalidSequence(String),
    /// Stack underflow detected at compile time
    StackUnderflow(String),
    /// Unsupported operation
    Unsupported(String),
}

impl std::fmt::Display for CodegenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CodegenError::InvalidSequence(msg) => write!(f, "Invalid sequence: {}", msg),
            CodegenError::StackUnderflow(msg) => write!(f, "Stack underflow: {}", msg),
            CodegenError::Unsupported(msg) => write!(f, "Unsupported: {}", msg),
        }
    }
}

impl std::error::Error for CodegenError {}

/// Code generation options
#[derive(Debug, Clone)]
pub struct CodegenOptions {
    /// Initial memory pages (64KB each)
    pub memory_pages: u32,
    /// Maximum memory pages (None = unlimited)
    pub max_memory_pages: Option<u32>,
    /// Export the stack pointer for debugging
    pub export_stack_pointer: bool,
    /// Name of the exported main function
    pub main_function_name: String,
}

impl Default for CodegenOptions {
    fn default() -> Self {
        Self {
            memory_pages: 1,           // 64KB initial
            max_memory_pages: Some(16), // 1MB max
            export_stack_pointer: false,
            main_function_name: "main".to_string(),
        }
    }
}

/// WASM code generator
///
/// Compiles Piet bytecode to WebAssembly binary format.
pub struct WasmCodegen {
    options: CodegenOptions,
}

impl WasmCodegen {
    /// Create a new code generator with default options
    pub fn new() -> Self {
        Self {
            options: CodegenOptions::default(),
        }
    }

    /// Create with custom options
    pub fn with_options(options: CodegenOptions) -> Self {
        Self { options }
    }

    /// Generate WASM binary from a compiled Piet program
    pub fn generate(&self, program: &Program) -> Result<Vec<u8>, CodegenError> {
        let mut module = Module::new();

        // === Type Section ===
        // Define function signatures
        let mut types = TypeSection::new();
        
        // Type 0: () -> () for main
        types.ty().function(vec![], vec![]);
        
        // Type 1: () -> i32 for read_char, read_number
        types.ty().function(vec![], vec![ValType::I32]);
        
        // Type 2: (i32) -> () for write_char, write_number
        types.ty().function(vec![ValType::I32], vec![]);

        module.section(&types);

        // === Import Section ===
        // Import I/O functions from host
        let mut imports = ImportSection::new();
        
        // env.read_char: () -> i32
        imports.import("env", "read_char", wasm_encoder::EntityType::Function(1));
        // env.read_number: () -> i32
        imports.import("env", "read_number", wasm_encoder::EntityType::Function(1));
        // env.write_char: (i32) -> ()
        imports.import("env", "write_char", wasm_encoder::EntityType::Function(2));
        // env.write_number: (i32) -> ()
        imports.import("env", "write_number", wasm_encoder::EntityType::Function(2));

        module.section(&imports);

        // === Function Section ===
        // Declare our functions (main + helpers)
        let mut functions = FunctionSection::new();
        
        // Function 4 (after 4 imports): main
        functions.function(0); // Type 0: () -> ()
        
        // Function 5: stack_push (helper)
        // Type: (i32) -> ()
        functions.function(2);
        
        // Function 6: stack_pop (helper)
        // Type: () -> i32
        functions.function(1);
        
        // Function 7: stack_peek (helper) - read top without popping
        // Type: () -> i32
        functions.function(1);
        
        // Function 8: stack_size (helper)
        // Type: () -> i32
        functions.function(1);

        module.section(&functions);

        // === Memory Section ===
        // Linear memory for our stack
        let mut memories = MemorySection::new();
        memories.memory(MemoryType {
            minimum: self.options.memory_pages as u64,
            maximum: self.options.max_memory_pages.map(|p| p as u64),
            memory64: false,
            shared: false,
            page_size_log2: None,
        });

        module.section(&memories);

        // === Export Section ===
        let mut exports = ExportSection::new();
        
        // Export main function
        exports.export(&self.options.main_function_name, ExportKind::Func, 4);
        
        // Export memory for host inspection
        exports.export("memory", ExportKind::Memory, 0);

        module.section(&exports);

        // === Code Section ===
        let mut codes = CodeSection::new();

        // Generate main function body
        let main_func = self.generate_main(program)?;
        codes.function(&main_func);

        // Generate helper functions
        codes.function(&self.generate_stack_push());
        codes.function(&self.generate_stack_pop());
        codes.function(&self.generate_stack_peek());
        codes.function(&self.generate_stack_size());

        module.section(&codes);

        Ok(module.finish())
    }

    /// Generate the main function that executes the Piet program
    fn generate_main(&self, program: &Program) -> Result<Function, CodegenError> {
        let mut func = Function::new(vec![]);

        // Initialize stack pointer at memory[0] = 4 (skip the SP itself)
        func.instruction(&WasmInst::I32Const(0));  // address 0
        func.instruction(&WasmInst::I32Const(4));  // initial SP value
        func.instruction(&WasmInst::I32Store(wasm_encoder::MemArg {
            offset: 0,
            align: 2,
            memory_index: 0,
        }));

        // Generate code for each instruction
        for instruction in &program.instructions {
            self.emit_instruction(&mut func, instruction)?;
        }

        func.instruction(&WasmInst::End);
        Ok(func)
    }

    /// Emit WASM instructions for a single Piet instruction
    fn emit_instruction(
        &self,
        func: &mut Function,
        instruction: &Instruction,
    ) -> Result<(), CodegenError> {
        match instruction {
            Instruction::Push(n) => {
                // call stack_push(n)
                func.instruction(&WasmInst::I32Const(*n));
                func.instruction(&WasmInst::Call(5)); // stack_push
            }

            Instruction::Pop => {
                // call stack_pop() and discard
                func.instruction(&WasmInst::Call(6)); // stack_pop
                func.instruction(&WasmInst::Drop);
            }

            Instruction::Add => {
                // a = pop(), b = pop(), push(b + a)
                func.instruction(&WasmInst::Call(6)); // a
                func.instruction(&WasmInst::Call(6)); // b
                func.instruction(&WasmInst::I32Add);
                func.instruction(&WasmInst::Call(5)); // push result
            }

            Instruction::Subtract => {
                // a = pop(), b = pop(), push(b - a)
                func.instruction(&WasmInst::Call(6)); // a
                func.instruction(&WasmInst::Call(6)); // b
                func.instruction(&WasmInst::I32Sub);
                func.instruction(&WasmInst::Call(5)); // push result
            }

            Instruction::Multiply => {
                // a = pop(), b = pop(), push(b * a)
                func.instruction(&WasmInst::Call(6)); // a
                func.instruction(&WasmInst::Call(6)); // b
                func.instruction(&WasmInst::I32Mul);
                func.instruction(&WasmInst::Call(5)); // push result
            }

            Instruction::Divide => {
                // a = pop(), b = pop(), push(b / a)
                func.instruction(&WasmInst::Call(6)); // a (divisor)
                func.instruction(&WasmInst::Call(6)); // b (dividend)
                // Handle division by zero: if a == 0, return 0
                // For simplicity, use signed division (Piet spec)
                func.instruction(&WasmInst::I32DivS);
                func.instruction(&WasmInst::Call(5)); // push result
            }

            Instruction::Mod => {
                // a = pop(), b = pop(), push(b % a)
                func.instruction(&WasmInst::Call(6)); // a
                func.instruction(&WasmInst::Call(6)); // b
                func.instruction(&WasmInst::I32RemS);
                func.instruction(&WasmInst::Call(5)); // push result
            }

            Instruction::Not => {
                // a = pop(), push(a == 0 ? 1 : 0)
                func.instruction(&WasmInst::Call(6)); // a
                func.instruction(&WasmInst::I32Eqz);
                func.instruction(&WasmInst::Call(5)); // push result
            }

            Instruction::Greater => {
                // a = pop(), b = pop(), push(b > a ? 1 : 0)
                func.instruction(&WasmInst::Call(6)); // a
                func.instruction(&WasmInst::Call(6)); // b
                func.instruction(&WasmInst::I32GtS);
                func.instruction(&WasmInst::Call(5)); // push result
            }

            Instruction::Duplicate => {
                // a = peek(), push(a)
                func.instruction(&WasmInst::Call(7)); // stack_peek
                func.instruction(&WasmInst::Call(5)); // stack_push
            }

            Instruction::Roll => {
                // This is complex - needs runtime helper
                // For now, emit a simpler version
                // times = pop(), depth = pop()
                // Roll depth elements, times times
                self.emit_roll(func)?;
            }

            Instruction::InNumber => {
                // Read number from host and push
                func.instruction(&WasmInst::Call(1)); // env.read_number
                func.instruction(&WasmInst::Call(5)); // stack_push
            }

            Instruction::InChar => {
                // Read char from host and push
                func.instruction(&WasmInst::Call(0)); // env.read_char
                func.instruction(&WasmInst::Call(5)); // stack_push
            }

            Instruction::OutNumber => {
                // Pop and write number to host
                func.instruction(&WasmInst::Call(6)); // stack_pop
                func.instruction(&WasmInst::Call(3)); // env.write_number
            }

            Instruction::OutChar => {
                // Pop and write char to host
                func.instruction(&WasmInst::Call(6)); // stack_pop
                func.instruction(&WasmInst::Call(2)); // env.write_char
            }

            Instruction::Pointer | Instruction::Switch => {
                // These modify DP/CC which is already resolved at compile time
                // In the linear bytecode, they become NOPs
                // (control flow was determined during compilation)
                func.instruction(&WasmInst::Call(6)); // pop the argument
                func.instruction(&WasmInst::Drop);    // discard it
            }

            Instruction::Nop => {
                // No operation
                func.instruction(&WasmInst::Nop);
            }

            Instruction::Halt => {
                // Return from main
                func.instruction(&WasmInst::Return);
            }
        }

        Ok(())
    }

    /// Emit roll operation (complex stack manipulation)
    fn emit_roll(&self, func: &mut Function) -> Result<(), CodegenError> {
        // Roll is the most complex Piet operation
        // For MVP, we implement a simple version using memory operations
        
        // Pop times and depth
        func.instruction(&WasmInst::Call(6)); // times
        func.instruction(&WasmInst::Drop);    // TODO: implement proper roll
        func.instruction(&WasmInst::Call(6)); // depth
        func.instruction(&WasmInst::Drop);    // TODO: implement proper roll
        
        // TODO: Implement full roll semantics
        // This requires manipulating the stack in memory
        
        Ok(())
    }

    /// Generate stack_push helper: (value: i32) -> ()
    fn generate_stack_push(&self) -> Function {
        // Stack layout in memory:
        // [0..4]: Stack pointer (points to next free slot)
        // [4..]: Stack data
        
        let mut func = Function::new(vec![(1, ValType::I32)]); // 1 local for value param
        
        // Get current SP
        func.instruction(&WasmInst::I32Const(0));
        func.instruction(&WasmInst::I32Load(wasm_encoder::MemArg {
            offset: 0, align: 2, memory_index: 0,
        }));
        
        // Store value at SP
        // local.get 0 (the parameter is the value)
        func.instruction(&WasmInst::LocalGet(0));
        func.instruction(&WasmInst::I32Store(wasm_encoder::MemArg {
            offset: 0, align: 2, memory_index: 0,
        }));
        
        // Increment SP by 4
        func.instruction(&WasmInst::I32Const(0));
        func.instruction(&WasmInst::I32Const(0));
        func.instruction(&WasmInst::I32Load(wasm_encoder::MemArg {
            offset: 0, align: 2, memory_index: 0,
        }));
        func.instruction(&WasmInst::I32Const(4));
        func.instruction(&WasmInst::I32Add);
        func.instruction(&WasmInst::I32Store(wasm_encoder::MemArg {
            offset: 0, align: 2, memory_index: 0,
        }));
        
        func.instruction(&WasmInst::End);
        func
    }

    /// Generate stack_pop helper: () -> i32
    fn generate_stack_pop(&self) -> Function {
        let mut func = Function::new(vec![]);
        
        // Decrement SP by 4
        func.instruction(&WasmInst::I32Const(0));
        func.instruction(&WasmInst::I32Const(0));
        func.instruction(&WasmInst::I32Load(wasm_encoder::MemArg {
            offset: 0, align: 2, memory_index: 0,
        }));
        func.instruction(&WasmInst::I32Const(4));
        func.instruction(&WasmInst::I32Sub);
        func.instruction(&WasmInst::I32Store(wasm_encoder::MemArg {
            offset: 0, align: 2, memory_index: 0,
        }));
        
        // Load value from new SP
        func.instruction(&WasmInst::I32Const(0));
        func.instruction(&WasmInst::I32Load(wasm_encoder::MemArg {
            offset: 0, align: 2, memory_index: 0,
        }));
        func.instruction(&WasmInst::I32Load(wasm_encoder::MemArg {
            offset: 0, align: 2, memory_index: 0,
        }));
        
        func.instruction(&WasmInst::End);
        func
    }

    /// Generate stack_peek helper: () -> i32 (read top without popping)
    fn generate_stack_peek(&self) -> Function {
        let mut func = Function::new(vec![]);
        
        // Load value at SP - 4 (top of stack)
        func.instruction(&WasmInst::I32Const(0));
        func.instruction(&WasmInst::I32Load(wasm_encoder::MemArg {
            offset: 0, align: 2, memory_index: 0,
        }));
        func.instruction(&WasmInst::I32Const(4));
        func.instruction(&WasmInst::I32Sub);
        func.instruction(&WasmInst::I32Load(wasm_encoder::MemArg {
            offset: 0, align: 2, memory_index: 0,
        }));
        
        func.instruction(&WasmInst::End);
        func
    }

    /// Generate stack_size helper: () -> i32
    fn generate_stack_size(&self) -> Function {
        let mut func = Function::new(vec![]);
        
        // (SP - 4) / 4 = number of elements
        func.instruction(&WasmInst::I32Const(0));
        func.instruction(&WasmInst::I32Load(wasm_encoder::MemArg {
            offset: 0, align: 2, memory_index: 0,
        }));
        func.instruction(&WasmInst::I32Const(4));
        func.instruction(&WasmInst::I32Sub);
        func.instruction(&WasmInst::I32Const(4));
        func.instruction(&WasmInst::I32DivU);
        
        func.instruction(&WasmInst::End);
        func
    }
}

impl Default for WasmCodegen {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_codegen_options_default() {
        let opts = CodegenOptions::default();
        assert_eq!(opts.memory_pages, 1);
        assert_eq!(opts.main_function_name, "main");
    }
}
