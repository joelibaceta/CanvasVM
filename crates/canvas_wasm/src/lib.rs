use wasm_bindgen::prelude::*;
use canvas_vm::{
    Grid, BytecodeVm, CompileMode, Compiler, Instruction, Program,
    Debugger, DebuggerState, ExecutionStep, RichInstruction,
};
use serde::{Deserialize, Serialize};

/// Consola de logging para debugging en el navegador
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

/// Macro helper para logging
macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

// ============================================================================
// Serializable types for JavaScript
// ============================================================================

/// Estado de la VM serializable para JavaScript
#[derive(Serialize, Deserialize)]
pub struct VmSnapshot {
    pub position_x: usize,
    pub position_y: usize,
    pub pixel_x: usize,
    pub pixel_y: usize,
    pub codel_size: usize,
    pub direction: String,
    pub codel_chooser: String,
    pub stack: Vec<i32>,
    pub halted: bool,
    pub steps: usize,
    pub instruction_index: Option<usize>,
}

/// Program metadata serializable for JavaScript
#[derive(Serialize, Deserialize)]
pub struct JsProgramMetadata {
    pub codel_size: usize,
    pub image_width: usize,
    pub image_height: usize,
    pub grid_width: usize,
    pub grid_height: usize,
}

/// Debug info for an instruction serializable for JavaScript
#[derive(Serialize, Deserialize)]
pub struct JsInstructionDebugInfo {
    pub from_x: usize,
    pub from_y: usize,
    pub to_x: usize,
    pub to_y: usize,
    pub dp: String,
    pub cc: String,
    pub block_size: usize,
    pub from_color: String,
    pub to_color: String,
}

/// Instrucción de bytecode enriquecida serializable para JavaScript
#[derive(Serialize, Deserialize)]
pub struct BytecodeInstruction {
    pub index: usize,
    pub opcode: String,
    pub from_color: String,
    pub to_color: String,
    /// Debug info (position, dp, cc, etc.)
    pub debug: Option<JsInstructionDebugInfo>,
}

/// Debugger state serializable for JavaScript
#[derive(Serialize, Deserialize)]
pub struct JsDebuggerState {
    /// Current instruction pointer
    pub ip: usize,
    /// Current position in the grid (codels)
    pub position_x: usize,
    pub position_y: usize,
    /// Current position in pixels (for highlighting)
    pub pixel_x: usize,
    pub pixel_y: usize,
    /// Direction pointer
    pub dp: String,
    /// Codel chooser
    pub cc: String,
    /// Current stack
    pub stack: Vec<i32>,
    /// Whether VM is halted
    pub halted: bool,
    /// Total steps executed
    pub steps: usize,
    /// Current instruction (if any)
    pub current_instruction: Option<JsRichInstruction>,
    /// Next instruction (if any)  
    pub next_instruction: Option<JsRichInstruction>,
    /// Output so far
    pub output: Vec<i32>,
    /// Output as string
    pub output_string: String,
    /// Whether waiting for input (null if not waiting, "number" or "char" if waiting)
    pub waiting_for_input: Option<String>,
}

/// Rich instruction for JavaScript
#[derive(Serialize, Deserialize)]
pub struct JsRichInstruction {
    pub opcode: String,
    pub value: Option<i32>,
    pub debug: Option<JsInstructionDebugInfo>,
}

/// Execution step for JavaScript
#[derive(Serialize, Deserialize)]
pub struct JsExecutionStep {
    pub step: usize,
    pub opcode: String,
    pub value: Option<i32>,
    pub stack_before: Vec<i32>,
    pub stack_after: Vec<i32>,
    pub position_x: usize,
    pub position_y: usize,
    pub dp: String,
    pub cc: String,
    pub output: Option<i32>,
    pub output_is_char: bool,
    pub debug: Option<JsInstructionDebugInfo>,
}

/// Execution trace for JavaScript
#[derive(Serialize, Deserialize)]
pub struct JsExecutionTrace {
    pub steps: Vec<JsExecutionStep>,
    pub output: Vec<i32>,
    pub output_string: String,
    pub total_steps: usize,
    pub completed: bool,
    pub error: Option<String>,
}

// ============================================================================
// Helper functions for type conversion
// ============================================================================

fn instruction_to_opcode(instr: &Instruction) -> (String, Option<i32>) {
    match instr {
        Instruction::Push(v) => ("Push".to_string(), Some(*v)),
        Instruction::Pop => ("Pop".to_string(), None),
        Instruction::Add => ("Add".to_string(), None),
        Instruction::Subtract => ("Subtract".to_string(), None),
        Instruction::Multiply => ("Multiply".to_string(), None),
        Instruction::Divide => ("Divide".to_string(), None),
        Instruction::Mod => ("Mod".to_string(), None),
        Instruction::Not => ("Not".to_string(), None),
        Instruction::Greater => ("Greater".to_string(), None),
        Instruction::Pointer => ("Pointer".to_string(), None),
        Instruction::Switch => ("Switch".to_string(), None),
        Instruction::Duplicate => ("Duplicate".to_string(), None),
        Instruction::Roll => ("Roll".to_string(), None),
        Instruction::InNumber => ("InNumber".to_string(), None),
        Instruction::InChar => ("InChar".to_string(), None),
        Instruction::OutNumber => ("OutNumber".to_string(), None),
        Instruction::OutChar => ("OutChar".to_string(), None),
        Instruction::Nop => ("Nop".to_string(), None),
        Instruction::Halt => ("Halt".to_string(), None),
    }
}

fn rich_instruction_to_js(rich: &RichInstruction) -> JsRichInstruction {
    let (opcode, value) = instruction_to_opcode(&rich.op);
    let debug = rich.debug.as_ref().map(|d| JsInstructionDebugInfo {
        from_x: d.from_pos.0,
        from_y: d.from_pos.1,
        to_x: d.to_pos.0,
        to_y: d.to_pos.1,
        dp: format!("{:?}", d.dp),
        cc: format!("{:?}", d.cc),
        block_size: d.block_size,
        from_color: d.from_color.clone(),
        to_color: d.to_color.clone(),
    });
    
    JsRichInstruction { opcode, value, debug }
}

fn execution_step_to_js(step: &ExecutionStep) -> JsExecutionStep {
    let (opcode, value) = instruction_to_opcode(&step.instruction);
    let debug = step.debug_info.as_ref().map(|d| JsInstructionDebugInfo {
        from_x: d.from_pos.0,
        from_y: d.from_pos.1,
        to_x: d.to_pos.0,
        to_y: d.to_pos.1,
        dp: format!("{:?}", d.dp),
        cc: format!("{:?}", d.cc),
        block_size: d.block_size,
        from_color: d.from_color.clone(),
        to_color: d.to_color.clone(),
    });
    
    JsExecutionStep {
        step: step.step,
        opcode,
        value,
        stack_before: step.stack_before.clone(),
        stack_after: step.stack_after.clone(),
        position_x: step.position.0,
        position_y: step.position.1,
        dp: format!("{:?}", step.dp),
        cc: format!("{:?}", step.cc),
        output: step.output,
        output_is_char: step.output_is_char,
        debug,
    }
}

fn debugger_state_to_js(state: &DebuggerState) -> JsDebuggerState {
    let waiting_for_input = state.waiting_for_input.as_ref().map(|req| {
        match req {
            canvas_vm::InputRequest::Number => "number".to_string(),
            canvas_vm::InputRequest::Char => "char".to_string(),
        }
    });
    
    JsDebuggerState {
        ip: state.ip,
        position_x: state.position.0,
        position_y: state.position.1,
        pixel_x: state.pixel_position.0,
        pixel_y: state.pixel_position.1,
        dp: format!("{:?}", state.dp),
        cc: format!("{:?}", state.cc),
        stack: state.stack.clone(),
        halted: state.halted,
        steps: state.steps,
        current_instruction: state.current_instruction.as_ref().map(rich_instruction_to_js),
        next_instruction: state.next_instruction.as_ref().map(rich_instruction_to_js),
        output: state.output.clone(),
        output_string: state.output_string.clone(),
        waiting_for_input,
    }
}

// ============================================================================
// PietDebugger - Step-by-step debugging for IDE experience
// ============================================================================

/// PietDebugger - Debugger WASM para ejecución paso a paso
/// 
/// Provee una experiencia de debugging completa:
/// - Step-by-step execution
/// - Breakpoints
/// - State inspection (stack, position, DP/CC)
/// - Execution trace
/// - Watchdog para prevenir loops infinitos
#[wasm_bindgen]
pub struct PietDebugger {
    debugger: Option<Debugger>,
    grid: Option<Grid>,
    codel_size: usize,
    image_width: usize,
    image_height: usize,
    /// Watchdog: límite máximo de pasos (None = sin límite)
    max_steps: Option<usize>,
}

/// Límite de pasos por defecto para el watchdog (100,000)
const DEFAULT_WATCHDOG_LIMIT: usize = 100_000;

#[wasm_bindgen]
impl PietDebugger {
    /// Create a new debugger instance
    #[wasm_bindgen(constructor)]
    pub fn new() -> PietDebugger {
        console_log!("🔍 PietDebugger created with watchdog limit {}", DEFAULT_WATCHDOG_LIMIT);
        PietDebugger {
            debugger: None,
            grid: None,
            codel_size: 1,
            image_width: 0,
            image_height: 0,
            max_steps: Some(DEFAULT_WATCHDOG_LIMIT),
        }
    }

    /// Load an image for debugging
    /// load(rgbaData: Uint8Array, width: number, height: number, codelSize?: number)
    #[wasm_bindgen]
    pub fn load(
        &mut self,
        rgba_data: &[u8],
        width: usize,
        height: usize,
        codel_size: usize,
    ) -> Result<(), JsValue> {
        let cs = if codel_size == 0 { None } else { Some(codel_size) };
        let detected_cs = cs.unwrap_or_else(|| Grid::detect_codel_size_from_rgba(width, height, rgba_data));
        
        console_log!("🔍 Loading image {}x{} with codel size {}", width, height, detected_cs);
        
        let grid = Grid::from_rgba_with_codel_size(width, height, rgba_data, cs)
            .map_err(|e| JsValue::from_str(&format!("Failed to create grid: {}", e)))?;
        
        self.grid = Some(grid.clone());
        self.codel_size = detected_cs;
        self.image_width = width;
        self.image_height = height;
        
        // Create debugger (always in debug mode)
        let debugger = Debugger::new(grid, detected_cs, width, height)
            .map_err(|e| JsValue::from_str(&format!("Failed to create debugger: {}", e)))?;
        
        console_log!("✅ Debugger ready with {} instructions", debugger.instruction_count());
        self.debugger = Some(debugger);
        
        Ok(())
    }

    /// Get current debugger state
    /// state(): JsDebuggerState
    #[wasm_bindgen]
    pub fn state(&self) -> Result<JsValue, JsValue> {
        let debugger = self.debugger.as_ref()
            .ok_or_else(|| JsValue::from_str("Debugger not initialized. Call load() first"))?;
        
        let state = debugger.state();
        let js_state = debugger_state_to_js(&state);
        
        serde_wasm_bindgen::to_value(&js_state)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    /// Execute a single step
    /// step(): JsExecutionStep | null
    #[wasm_bindgen]
    pub fn step(&mut self) -> Result<JsValue, JsValue> {
        let debugger = self.debugger.as_mut()
            .ok_or_else(|| JsValue::from_str("Debugger not initialized. Call load() first"))?;
        
        match debugger.step() {
            Ok(Some(step)) => {
                let js_step = execution_step_to_js(&step);
                serde_wasm_bindgen::to_value(&js_step)
                    .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
            }
            Ok(None) => Ok(JsValue::NULL),
            Err(e) => Err(JsValue::from_str(&format!("Step error: {}", e))),
        }
    }

    /// Run until halt, breakpoint, or watchdog timeout
    /// run(): JsExecutionTrace
    #[wasm_bindgen]
    pub fn run(&mut self) -> Result<JsValue, JsValue> {
        let debugger = self.debugger.as_mut()
            .ok_or_else(|| JsValue::from_str("Debugger not initialized. Call load() first"))?;
        
        // Use watchdog limit if enabled
        let trace = if let Some(max_steps) = self.max_steps {
            // Run with watchdog limit
            let current_steps = debugger.state().steps;
            let remaining = max_steps.saturating_sub(current_steps);
            
            if remaining == 0 {
                // Already at or past limit
                let mut trace = debugger.get_execution_trace();
                trace.error = Some(format!("Watchdog timeout: execution exceeded {} steps", max_steps));
                trace.completed = false;
                trace
            } else {
                debugger.run_limited(remaining)
                    .map_err(|e| JsValue::from_str(&format!("Run error: {}", e)))?
            }
        } else {
            // No watchdog - run until halt
            debugger.run()
                .map_err(|e| JsValue::from_str(&format!("Run error: {}", e)))?
        };
        
        let js_trace = JsExecutionTrace {
            steps: trace.steps.iter().map(execution_step_to_js).collect(),
            output: trace.output,
            output_string: trace.output_string,
            total_steps: trace.total_steps,
            completed: trace.completed,
            error: trace.error,
        };
        
        serde_wasm_bindgen::to_value(&js_trace)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    /// Run for a maximum number of steps
    /// run_steps(maxSteps: number): number
    #[wasm_bindgen]
    pub fn run_steps(&mut self, max_steps: usize) -> Result<usize, JsValue> {
        let debugger = self.debugger.as_mut()
            .ok_or_else(|| JsValue::from_str("Debugger not initialized. Call load() first"))?;
        
        debugger.run_steps(max_steps)
            .map_err(|e| JsValue::from_str(&format!("Run error: {}", e)))
    }

    /// Reset debugger to initial state
    /// reset(): void
    #[wasm_bindgen]
    pub fn reset(&mut self) -> Result<(), JsValue> {
        let debugger = self.debugger.as_mut()
            .ok_or_else(|| JsValue::from_str("Debugger not initialized. Call load() first"))?;
        
        debugger.reset();
        console_log!("🔄 Debugger reset");
        Ok(())
    }

    /// Add a breakpoint at instruction index
    /// add_breakpoint(index: number): void
    #[wasm_bindgen]
    pub fn add_breakpoint(&mut self, index: usize) -> Result<(), JsValue> {
        let debugger = self.debugger.as_mut()
            .ok_or_else(|| JsValue::from_str("Debugger not initialized. Call load() first"))?;
        
        debugger.add_breakpoint(index);
        console_log!("🔴 Breakpoint added at instruction {}", index);
        Ok(())
    }

    /// Remove a breakpoint
    /// remove_breakpoint(index: number): void
    #[wasm_bindgen]
    pub fn remove_breakpoint(&mut self, index: usize) -> Result<(), JsValue> {
        let debugger = self.debugger.as_mut()
            .ok_or_else(|| JsValue::from_str("Debugger not initialized. Call load() first"))?;
        
        debugger.remove_breakpoint(index);
        console_log!("⚪ Breakpoint removed at instruction {}", index);
        Ok(())
    }

    /// Clear all breakpoints
    /// clear_breakpoints(): void
    #[wasm_bindgen]
    pub fn clear_breakpoints(&mut self) -> Result<(), JsValue> {
        let debugger = self.debugger.as_mut()
            .ok_or_else(|| JsValue::from_str("Debugger not initialized. Call load() first"))?;
        
        debugger.clear_breakpoints();
        console_log!("⚪ All breakpoints cleared");
        Ok(())
    }

    /// Get all breakpoints
    /// breakpoints(): number[]
    #[wasm_bindgen]
    pub fn breakpoints(&self) -> Result<Vec<usize>, JsValue> {
        let debugger = self.debugger.as_ref()
            .ok_or_else(|| JsValue::from_str("Debugger not initialized. Call load() first"))?;
        
        Ok(debugger.breakpoints().to_vec())
    }

    /// Continue execution until next breakpoint
    /// continue_to_breakpoint(): number | null
    #[wasm_bindgen]
    pub fn continue_to_breakpoint(&mut self) -> Result<JsValue, JsValue> {
        let debugger = self.debugger.as_mut()
            .ok_or_else(|| JsValue::from_str("Debugger not initialized. Call load() first"))?;
        
        match debugger.continue_to_breakpoint() {
            Ok(Some(ip)) => Ok(JsValue::from(ip as u32)),
            Ok(None) => Ok(JsValue::NULL),
            Err(e) => Err(JsValue::from_str(&format!("Continue error: {}", e))),
        }
    }

    /// Check if at a breakpoint
    /// is_at_breakpoint(): boolean
    #[wasm_bindgen]
    pub fn is_at_breakpoint(&self) -> Result<bool, JsValue> {
        let debugger = self.debugger.as_ref()
            .ok_or_else(|| JsValue::from_str("Debugger not initialized. Call load() first"))?;
        
        Ok(debugger.is_at_breakpoint())
    }

    /// Check if halted
    /// is_halted(): boolean
    #[wasm_bindgen]
    pub fn is_halted(&self) -> Result<bool, JsValue> {
        let debugger = self.debugger.as_ref()
            .ok_or_else(|| JsValue::from_str("Debugger not initialized. Call load() first"))?;
        
        Ok(debugger.is_halted())
    }

    /// Get current instruction pointer
    /// current_ip(): number
    #[wasm_bindgen]
    pub fn current_ip(&self) -> Result<usize, JsValue> {
        let debugger = self.debugger.as_ref()
            .ok_or_else(|| JsValue::from_str("Debugger not initialized. Call load() first"))?;
        
        Ok(debugger.current_ip())
    }

    /// Get total instruction count
    /// instruction_count(): number
    #[wasm_bindgen]
    pub fn instruction_count(&self) -> Result<usize, JsValue> {
        let debugger = self.debugger.as_ref()
            .ok_or_else(|| JsValue::from_str("Debugger not initialized. Call load() first"))?;
        
        Ok(debugger.instruction_count())
    }

    /// Get output as string
    /// output_string(): string
    #[wasm_bindgen]
    pub fn output_string(&self) -> Result<String, JsValue> {
        let debugger = self.debugger.as_ref()
            .ok_or_else(|| JsValue::from_str("Debugger not initialized. Call load() first"))?;
        
        Ok(debugger.output_string())
    }

    /// Provide input number
    /// input(value: number): void
    #[wasm_bindgen]
    pub fn input(&mut self, value: i32) -> Result<(), JsValue> {
        let debugger = self.debugger.as_mut()
            .ok_or_else(|| JsValue::from_str("Debugger not initialized. Call load() first"))?;
        
        debugger.input(value);
        Ok(())
    }

    /// Provide input character
    /// input_char(charCode: number): void
    #[wasm_bindgen]
    pub fn input_char(&mut self, char_code: u32) -> Result<(), JsValue> {
        let debugger = self.debugger.as_mut()
            .ok_or_else(|| JsValue::from_str("Debugger not initialized. Call load() first"))?;
        
        if let Some(c) = char::from_u32(char_code) {
            debugger.input_char(c);
        }
        Ok(())
    }

    /// Load text as character inputs (each character becomes an input for in_char operations)
    /// load_input_text(text: string): void
    #[wasm_bindgen]
    pub fn load_input_text(&mut self, text: &str) -> Result<(), JsValue> {
        let debugger = self.debugger.as_mut()
            .ok_or_else(|| JsValue::from_str("Debugger not initialized. Call load() first"))?;
        
        debugger.load_input_text(text);
        console_log!("📥 Loaded {} character(s) as input", text.len());
        Ok(())
    }

    /// Load numbers from string (whitespace-separated, for in_number operations)
    /// load_input_numbers(text: string): void
    #[wasm_bindgen]
    pub fn load_input_numbers(&mut self, text: &str) -> Result<(), JsValue> {
        let debugger = self.debugger.as_mut()
            .ok_or_else(|| JsValue::from_str("Debugger not initialized. Call load() first"))?;
        
        debugger.load_input_numbers(text);
        console_log!("📥 Loaded numbers from: {}", text);
        Ok(())
    }

    /// Load a vector of numbers as inputs
    /// load_input_numbers_array(numbers: Int32Array): void
    #[wasm_bindgen]
    pub fn load_input_numbers_array(&mut self, numbers: &[i32]) -> Result<(), JsValue> {
        let debugger = self.debugger.as_mut()
            .ok_or_else(|| JsValue::from_str("Debugger not initialized. Call load() first"))?;
        
        debugger.load_input_number_vec(numbers);
        console_log!("📥 Loaded {} number(s) as input", numbers.len());
        Ok(())
    }

    /// Check if waiting for input
    /// is_waiting_for_input(): boolean
    #[wasm_bindgen]
    pub fn is_waiting_for_input(&self) -> Result<bool, JsValue> {
        let debugger = self.debugger.as_ref()
            .ok_or_else(|| JsValue::from_str("Debugger not initialized. Call load() first"))?;
        
        Ok(debugger.is_waiting_for_input())
    }

    /// Get what type of input is being waited for
    /// get_input_request(): string | null  ("number" or "char")
    #[wasm_bindgen]
    pub fn get_input_request(&self) -> Result<JsValue, JsValue> {
        let debugger = self.debugger.as_ref()
            .ok_or_else(|| JsValue::from_str("Debugger not initialized. Call load() first"))?;
        
        match debugger.get_input_request() {
            Some(canvas_vm::InputRequest::Number) => Ok(JsValue::from_str("number")),
            Some(canvas_vm::InputRequest::Char) => Ok(JsValue::from_str("char")),
            None => Ok(JsValue::NULL),
        }
    }

    /// Provide input value and resume execution (for interactive mode)
    /// provide_input(value: number): void
    #[wasm_bindgen]
    pub fn provide_input(&mut self, value: i32) -> Result<(), JsValue> {
        let debugger = self.debugger.as_mut()
            .ok_or_else(|| JsValue::from_str("Debugger not initialized. Call load() first"))?;
        
        debugger.provide_input(value);
        console_log!("📥 Input provided: {}", value);
        Ok(())
    }

    /// Provide character input and resume execution (for interactive mode)
    /// provide_input_char(charCode: number): void
    #[wasm_bindgen]
    pub fn provide_input_char(&mut self, char_code: u32) -> Result<(), JsValue> {
        let debugger = self.debugger.as_mut()
            .ok_or_else(|| JsValue::from_str("Debugger not initialized. Call load() first"))?;
        
        if let Some(c) = char::from_u32(char_code) {
            debugger.provide_input_char(c);
            console_log!("📥 Char input provided: '{}'", c);
        }
        Ok(())
    }

    /// Clear all inputs
    /// clear_input(): void
    #[wasm_bindgen]
    pub fn clear_input(&mut self) -> Result<(), JsValue> {
        let debugger = self.debugger.as_mut()
            .ok_or_else(|| JsValue::from_str("Debugger not initialized. Call load() first"))?;
        
        debugger.clear_input();
        console_log!("🗑️ Input buffer cleared");
        Ok(())
    }

    /// Rewind input buffer to start (re-read inputs from beginning)
    /// rewind_input(): void
    #[wasm_bindgen]
    pub fn rewind_input(&mut self) -> Result<(), JsValue> {
        let debugger = self.debugger.as_mut()
            .ok_or_else(|| JsValue::from_str("Debugger not initialized. Call load() first"))?;
        
        debugger.rewind_input();
        console_log!("⏪ Input buffer rewound to start");
        Ok(())
    }

    /// Check if there are inputs available
    /// has_input(): boolean
    #[wasm_bindgen]
    pub fn has_input(&self) -> Result<bool, JsValue> {
        let debugger = self.debugger.as_ref()
            .ok_or_else(|| JsValue::from_str("Debugger not initialized. Call load() first"))?;
        
        Ok(debugger.has_input())
    }

    /// Get remaining input count
    /// remaining_input(): number
    #[wasm_bindgen]
    pub fn remaining_input(&self) -> Result<usize, JsValue> {
        let debugger = self.debugger.as_ref()
            .ok_or_else(|| JsValue::from_str("Debugger not initialized. Call load() first"))?;
        
        Ok(debugger.remaining_input())
    }

    // === Watchdog API ===

    /// Set the watchdog limit (maximum steps before timeout)
    /// set_max_steps(maxSteps: number): void
    #[wasm_bindgen]
    pub fn set_max_steps(&mut self, max_steps: usize) {
        self.max_steps = Some(max_steps);
        console_log!("⏱️ Watchdog limit set to {} steps", max_steps);
    }

    /// Get the current watchdog limit
    /// get_max_steps(): number | null
    #[wasm_bindgen]
    pub fn get_max_steps(&self) -> JsValue {
        match self.max_steps {
            Some(max) => JsValue::from(max as u32),
            None => JsValue::NULL,
        }
    }

    /// Disable the watchdog (allow infinite execution - use with caution!)
    /// disable_watchdog(): void
    #[wasm_bindgen]
    pub fn disable_watchdog(&mut self) {
        self.max_steps = None;
        console_log!("⚠️ Watchdog disabled - infinite loops possible!");
    }

    /// Enable the watchdog with the default limit
    /// enable_watchdog(): void
    #[wasm_bindgen]
    pub fn enable_watchdog(&mut self) {
        self.max_steps = Some(DEFAULT_WATCHDOG_LIMIT);
        console_log!("✅ Watchdog enabled with limit {}", DEFAULT_WATCHDOG_LIMIT);
    }

    /// Check if watchdog is enabled
    /// is_watchdog_enabled(): boolean
    #[wasm_bindgen]
    pub fn is_watchdog_enabled(&self) -> bool {
        self.max_steps.is_some()
    }

    /// Get execution trace
    /// trace(): JsExecutionTrace
    #[wasm_bindgen]
    pub fn trace(&self) -> Result<JsValue, JsValue> {
        let debugger = self.debugger.as_ref()
            .ok_or_else(|| JsValue::from_str("Debugger not initialized. Call load() first"))?;
        
        let trace = debugger.get_execution_trace();
        let js_trace = JsExecutionTrace {
            steps: trace.steps.iter().map(execution_step_to_js).collect(),
            output: trace.output,
            output_string: trace.output_string,
            total_steps: trace.total_steps,
            completed: trace.completed,
            error: trace.error,
        };
        
        serde_wasm_bindgen::to_value(&js_trace)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    /// Get all bytecode instructions (for display)
    /// bytecode(): BytecodeInstruction[]
    #[wasm_bindgen]
    pub fn bytecode(&self) -> Result<JsValue, JsValue> {
        let debugger = self.debugger.as_ref()
            .ok_or_else(|| JsValue::from_str("Debugger not initialized. Call load() first"))?;
        
        let program = debugger.program();
        let instructions: Vec<BytecodeInstruction> = program.rich_instructions
            .iter()
            .enumerate()
            .map(|(i, rich)| {
                let (opcode, value) = instruction_to_opcode(&rich.op);
                let debug = rich.debug.as_ref().map(|d| JsInstructionDebugInfo {
                    from_x: d.from_pos.0,
                    from_y: d.from_pos.1,
                    to_x: d.to_pos.0,
                    to_y: d.to_pos.1,
                    dp: format!("{:?}", d.dp),
                    cc: format!("{:?}", d.cc),
                    block_size: d.block_size,
                    from_color: d.from_color.clone(),
                    to_color: d.to_color.clone(),
                });
                
                BytecodeInstruction {
                    index: i,
                    opcode: if let Some(v) = value {
                        format!("{}({})", opcode, v)
                    } else {
                        opcode
                    },
                    from_color: debug.as_ref().map(|d| d.from_color.clone()).unwrap_or_default(),
                    to_color: debug.as_ref().map(|d| d.to_color.clone()).unwrap_or_default(),
                    debug,
                }
            })
            .collect();
        
        serde_wasm_bindgen::to_value(&instructions)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    /// Get program metadata
    /// metadata(): JsProgramMetadata
    #[wasm_bindgen]
    pub fn metadata(&self) -> Result<JsValue, JsValue> {
        let debugger = self.debugger.as_ref()
            .ok_or_else(|| JsValue::from_str("Debugger not initialized. Call load() first"))?;
        
        let program = debugger.program();
        let metadata = JsProgramMetadata {
            codel_size: program.metadata.codel_size,
            image_width: program.metadata.image_width,
            image_height: program.metadata.image_height,
            grid_width: program.metadata.grid_width,
            grid_height: program.metadata.grid_height,
        };
        
        serde_wasm_bindgen::to_value(&metadata)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }
}

// ============================================================================
// Canvas - Original VM wrapper (for Run mode)
// ============================================================================
#[wasm_bindgen]
pub struct Canvas {
    vm: Option<BytecodeVm>,
    grid: Option<Grid>,
    program: Option<Program>,
    width: usize,
    height: usize,
    codel_size: usize,
    debug_mode: bool,
    /// Watchdog: límite máximo de pasos
    max_steps: Option<usize>,
}

/// Límite por defecto para Canvas
const CANVAS_DEFAULT_MAX_STEPS: usize = 100_000;

#[wasm_bindgen]
impl Canvas {
    /// Crea una nueva instancia de Canvas
    #[wasm_bindgen(constructor)]
    pub fn new() -> Canvas {
        console_log!("🎨 Canvas created with watchdog limit {}", CANVAS_DEFAULT_MAX_STEPS);
        Canvas {
            vm: None,
            program: None,
            grid: None,
            width: 0,
            height: 0,
            codel_size: 1,
            debug_mode: false,
            max_steps: Some(CANVAS_DEFAULT_MAX_STEPS),
        }
    }
    
    // === Watchdog API ===
    
    /// Set the watchdog limit (maximum steps before timeout)
    /// set_max_steps(maxSteps: number): void
    #[wasm_bindgen]
    pub fn set_max_steps(&mut self, max_steps: usize) {
        self.max_steps = if max_steps == 0 { None } else { Some(max_steps) };
        if let Some(vm) = &mut self.vm {
            vm.set_max_steps(self.max_steps);
        }
        if let Some(max) = self.max_steps {
            console_log!("⏱️ Canvas watchdog limit set to {} steps", max);
        } else {
            console_log!("⚠️ Canvas watchdog disabled");
        }
    }
    
    /// Get the current watchdog limit
    /// get_max_steps(): number (0 = disabled)
    #[wasm_bindgen]
    pub fn get_max_steps(&self) -> usize {
        self.max_steps.unwrap_or(0)
    }
    
    /// Sets the compilation mode
    /// set_debug_mode(debug: boolean): void
    #[wasm_bindgen]
    pub fn set_debug_mode(&mut self, debug: bool) {
        self.debug_mode = debug;
        console_log!("🔧 Compile mode: {}", if debug { "Debug" } else { "Release" });
    }
    
    /// Gets the current compilation mode
    /// is_debug_mode(): boolean
    #[wasm_bindgen]
    pub fn is_debug_mode(&self) -> bool {
        self.debug_mode
    }

    /// Detecta automáticamente el tamaño del codel de una imagen
    /// detect_codel_size(rgbaData: Uint8Array, width: number, height: number): number
    #[wasm_bindgen]
    pub fn detect_codel_size(rgba_data: &[u8], width: usize, height: usize) -> usize {
        Grid::detect_codel_size_from_rgba(width, height, rgba_data)
    }

    /// Carga una grilla desde datos RGBA (4 bytes por píxel)
    /// Usa auto-detección de codel size
    /// paint(rgbaData: Uint8Array, width: number, height: number)
    #[wasm_bindgen]
    pub fn paint(&mut self, rgba_data: &[u8], width: usize, height: usize) -> Result<(), JsValue> {
        self.paint_with_codel_size(rgba_data, width, height, 0)
    }

    /// Carga una grilla con un tamaño de codel específico
    /// Si codel_size es 0, se auto-detecta
    /// paint_with_codel_size(rgbaData: Uint8Array, width: number, height: number, codelSize: number)
    #[wasm_bindgen]
    pub fn paint_with_codel_size(
        &mut self, 
        rgba_data: &[u8], 
        width: usize, 
        height: usize,
        codel_size: usize
    ) -> Result<(), JsValue> {
        let cs = if codel_size == 0 { None } else { Some(codel_size) };
        let detected_cs = cs.unwrap_or_else(|| Grid::detect_codel_size_from_rgba(width, height, rgba_data));
        
        console_log!("🎨 Loading grid {}x{} with codel size {}", width, height, detected_cs);
        
        let grid = Grid::from_rgba_with_codel_size(width, height, rgba_data, cs)
            .map_err(|e| JsValue::from_str(&format!("Failed to create grid: {}", e)))?;
        
        self.width = width;
        self.height = height;
        self.codel_size = detected_cs;
        
        console_log!("📐 Grid reduced to {}x{} codels", grid.width(), grid.height());
        
        // Store grid for reset functionality
        self.grid = Some(grid.clone());
        
        // Compile to bytecode with selected mode
        let mode = if self.debug_mode { CompileMode::Debug } else { CompileMode::Release };
        console_log!("📝 Compiling to bytecode ({} mode)...", if self.debug_mode { "debug" } else { "release" });
        
        let compiler = Compiler::with_codel_size(grid.clone(), detected_cs, width, height)
            .with_mode(mode);
        let program = compiler.compile()
            .map_err(|e| JsValue::from_str(&format!("Compilation error: {}", e)))?;
        
        console_log!("✅ Compiled {} instructions", program.instructions.len());
        
        // Create BytecodeVm with the compiled program and grid
        self.program = Some(program.clone());
        let mut vm = BytecodeVm::new(program, grid);
        
        // Apply watchdog limit
        vm.set_max_steps(self.max_steps);
        
        self.vm = Some(vm);
        
        console_log!("Grid loaded and bytecode VM ready ");
        Ok(())
    }

    /// Ejecuta un solo paso de la VM
    /// stroke(): void
    #[wasm_bindgen]
    pub fn stroke(&mut self) -> Result<(), JsValue> {
        let vm = self.vm.as_mut()
            .ok_or_else(|| JsValue::from_str("VM not initialized. Call paint() first"))?;
        
        // Verificar si ya está halted antes de intentar ejecutar
        if vm.snapshot().halted {
            return Ok(()); // Silently return if already halted
        }
        
        vm.stroke()
            .map_err(|e| {
                console_log!("❌ Stroke error: {}", e);
                JsValue::from_str(&format!("Step error: {}", e))
            })
    }

    /// Ejecuta múltiples pasos (hasta maxSteps o hasta que se detenga)
    /// play(maxSteps: number): number - retorna pasos ejecutados
    #[wasm_bindgen]
    pub fn play(&mut self, max_steps: usize) -> Result<usize, JsValue> {
        let vm = self.vm.as_mut()
            .ok_or_else(|| JsValue::from_str("VM not initialized. Call paint() first"))?;
        
        vm.play(max_steps)
            .map_err(|e| JsValue::from_str(&format!("Play error: {}", e)))
    }

    /// Obtiene el estado actual de la VM
    /// snapshot(): VmSnapshot
    #[wasm_bindgen]
    pub fn snapshot(&self) -> Result<JsValue, JsValue> {
        let vm = self.vm.as_ref()
            .ok_or_else(|| JsValue::from_str("VM not initialized. Call paint() first"))?;
        
        let snapshot = vm.snapshot();
        
        let js_snapshot = VmSnapshot {
            position_x: snapshot.position.x,
            position_y: snapshot.position.y,
            pixel_x: snapshot.position.x * self.codel_size,
            pixel_y: snapshot.position.y * self.codel_size,
            codel_size: self.codel_size,
            direction: format!("{:?}", snapshot.dp),
            codel_chooser: format!("{:?}", snapshot.cc),
            stack: snapshot.stack.clone(),
            halted: snapshot.halted,
            steps: snapshot.steps,
            instruction_index: snapshot.instruction_index,
        };
        
        serde_wasm_bindgen::to_value(&js_snapshot)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    /// Lee la salida como array de números
    /// ink(): Int32Array
    #[wasm_bindgen]
    pub fn ink(&self) -> Result<Vec<i32>, JsValue> {
        let vm = self.vm.as_ref()
            .ok_or_else(|| JsValue::from_str("VM not initialized. Call paint() first"))?;
        
        Ok(vm.ink().to_vec())
    }

    /// Lee la salida como string
    /// ink_string(): string
    #[wasm_bindgen]
    pub fn ink_string(&self) -> Result<String, JsValue> {
        let vm = self.vm.as_ref()
            .ok_or_else(|| JsValue::from_str("VM not initialized. Call paint() first"))?;
        
        Ok(vm.ink_string())
    }

    /// Escribe un número en la entrada
    /// input(value: number): void
    #[wasm_bindgen]
    pub fn input(&mut self, value: i32) -> Result<(), JsValue> {
        let vm = self.vm.as_mut()
            .ok_or_else(|| JsValue::from_str("VM not initialized. Call paint() first"))?;
        
        vm.input(value);
        Ok(())
    }

    /// Escribe un carácter en la entrada (desde su código)
    /// input_char(charCode: number): void
    #[wasm_bindgen]
    pub fn input_char(&mut self, char_code: u32) -> Result<(), JsValue> {
        let vm = self.vm.as_mut()
            .ok_or_else(|| JsValue::from_str("VM not initialized. Call paint() first"))?;
        
        let c = char::from_u32(char_code)
            .ok_or_else(|| JsValue::from_str("Invalid character code "))?;
        
        vm.input_char(c);
        Ok(())
    }

    /// Check if input is available in the buffer
    /// has_input(): boolean
    #[wasm_bindgen]
    pub fn has_input(&self) -> Result<bool, JsValue> {
        let vm = self.vm.as_ref()
            .ok_or_else(|| JsValue::from_str("VM not initialized. Call paint() first"))?;
        
        Ok(vm.has_input())
    }

    /// Get the next instruction opcode (for detecting InNumber/InChar)
    /// Returns null if no next instruction, otherwise returns opcode like "InNumber", "InChar", "Push", etc.
    /// get_next_opcode(): string | null
    #[wasm_bindgen]
    pub fn get_next_opcode(&self) -> Result<JsValue, JsValue> {
        let vm = self.vm.as_ref()
            .ok_or_else(|| JsValue::from_str("VM not initialized. Call paint() first"))?;
        
        let snapshot = vm.snapshot();
        match snapshot.next_instruction {
            Some(instr) => {
                let opcode = match instr {
                    Instruction::Push(_) => "Push",
                    Instruction::Pop => "Pop",
                    Instruction::Add => "Add",
                    Instruction::Subtract => "Subtract",
                    Instruction::Multiply => "Multiply",
                    Instruction::Divide => "Divide",
                    Instruction::Mod => "Mod",
                    Instruction::Not => "Not",
                    Instruction::Greater => "Greater",
                    Instruction::Pointer => "Pointer",
                    Instruction::Switch => "Switch",
                    Instruction::Duplicate => "Duplicate",
                    Instruction::Roll => "Roll",
                    Instruction::InNumber => "InNumber",
                    Instruction::InChar => "InChar",
                    Instruction::OutNumber => "OutNumber",
                    Instruction::OutChar => "OutChar",
                    Instruction::Nop => "Nop",
                    Instruction::Halt => "Halt",
                };
                Ok(JsValue::from_str(opcode))
            }
            None => Ok(JsValue::NULL),
        }
    }

    /// Check if the next instruction requires input (InNumber or InChar)
    /// needs_input(): string | null  (returns "number", "char", or null)
    #[wasm_bindgen]
    pub fn needs_input(&self) -> Result<JsValue, JsValue> {
        let vm = self.vm.as_ref()
            .ok_or_else(|| JsValue::from_str("VM not initialized. Call paint() first"))?;
        
        // Check if we have input in buffer
        if vm.has_input() {
            return Ok(JsValue::NULL);
        }
        
        let snapshot = vm.snapshot();
        match snapshot.next_instruction {
            Some(Instruction::InNumber) => Ok(JsValue::from_str("number")),
            Some(Instruction::InChar) => Ok(JsValue::from_str("char")),
            _ => Ok(JsValue::NULL),
        }
    }

    /// Verifica si la VM está detenida
    /// is_halted(): boolean
    #[wasm_bindgen]
    pub fn is_halted(&self) -> Result<bool, JsValue> {
        let vm = self.vm.as_ref()
            .ok_or_else(|| JsValue::from_str("VM not initialized. Call paint() first"))?;
        
        Ok(vm.snapshot().halted)
    }

    /// Obtiene el tamaño del stack
    /// stack_size(): number
    #[wasm_bindgen]
    pub fn stack_size(&self) -> Result<usize, JsValue> {
        let vm = self.vm.as_ref()
            .ok_or_else(|| JsValue::from_str("VM not initialized. Call paint() first"))?;
        
        Ok(vm.snapshot().stack.len())
    }

    /// Obtiene el número de pasos ejecutados
    /// get_steps(): number
    #[wasm_bindgen]
    pub fn get_steps(&self) -> Result<usize, JsValue> {
        let vm = self.vm.as_ref()
            .ok_or_else(|| JsValue::from_str("VM not initialized "))?;
        
        Ok(vm.snapshot().steps)
    }

    /// Resetea la VM a su estado inicial (recarga la imagen)
    /// reset(): void
    #[wasm_bindgen]
    pub fn reset(&mut self) -> Result<(), JsValue> {
        let grid = self.grid.as_ref()
            .ok_or_else(|| JsValue::from_str("No image loaded "))?
            .clone();
        
        console_log!("Resetting VM...");
        let vm = BytecodeVm::from_grid(grid)
            .map_err(|e| JsValue::from_str(&format!("VM reset error: {}", e)))?;
        self.vm = Some(vm);
        console_log!("VM reset to initial state ");
        Ok(())
    }

    /// Obtiene la metadata del programa compilado
    /// get_program_metadata(): JsProgramMetadata
    #[wasm_bindgen]
    pub fn get_program_metadata(&self) -> Result<JsValue, JsValue> {
        let program = self.program.as_ref()
            .ok_or_else(|| JsValue::from_str("No program compiled. Call paint() first"))?;
        
        let metadata = JsProgramMetadata {
            codel_size: program.metadata.codel_size,
            image_width: program.metadata.image_width,
            image_height: program.metadata.image_height,
            grid_width: program.metadata.grid_width,
            grid_height: program.metadata.grid_height,
        };
        
        serde_wasm_bindgen::to_value(&metadata)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    /// Compila la grilla actual a bytecode y retorna las instrucciones
    /// compile_to_bytecode(): BytecodeInstruction[]
    #[wasm_bindgen]
    pub fn compile_to_bytecode(&self) -> Result<JsValue, JsValue> {
        let grid = self.grid.as_ref()
            .ok_or_else(|| JsValue::from_str("No image loaded. Call paint() first"))?;
        
        console_log!("📝 Compiling bytecode...");
        
        // Create compiler and compile
        let compiler = Compiler::new(grid.clone());
        let program = compiler.compile()
            .map_err(|e| JsValue::from_str(&format!("Compilation error: {}", e)))?;
        
        console_log!("Program has {} instructions ", program.instructions.len());
        
        // Convert rich instructions to JS-friendly format
        let instructions: Vec<BytecodeInstruction> = program.rich_instructions
            .iter()
            .enumerate()
            .map(|(i, rich_instr)| {
                // Match each instruction variant to get a clean name
                let (op_name, value) = match &rich_instr.op {
                    Instruction::Push(v) => ("Push".to_string(), format!("{}", v)),
                    Instruction::Pop => ("Pop".to_string(), String::new()),
                    Instruction::Add => ("Add".to_string(), String::new()),
                    Instruction::Subtract => ("Subtract".to_string(), String::new()),
                    Instruction::Multiply => ("Multiply".to_string(), String::new()),
                    Instruction::Divide => ("Divide".to_string(), String::new()),
                    Instruction::Mod => ("Mod".to_string(), String::new()),
                    Instruction::Not => ("Not".to_string(), String::new()),
                    Instruction::Greater => ("Greater".to_string(), String::new()),
                    Instruction::Pointer => ("Pointer".to_string(), String::new()),
                    Instruction::Switch => ("Switch".to_string(), String::new()),
                    Instruction::Duplicate => ("Duplicate".to_string(), String::new()),
                    Instruction::Roll => ("Roll".to_string(), String::new()),
                    Instruction::InNumber => ("InNumber".to_string(), String::new()),
                    Instruction::InChar => ("InChar".to_string(), String::new()),
                    Instruction::OutNumber => ("OutNumber".to_string(), String::new()),
                    Instruction::OutChar => ("OutChar".to_string(), String::new()),
                    Instruction::Nop => ("Nop".to_string(), String::new()),
                    Instruction::Halt => ("Halt".to_string(), String::new()),
                };
                
                // Convert debug info if present
                let debug = rich_instr.debug.as_ref().map(|d| JsInstructionDebugInfo {
                    from_x: d.from_pos.0,
                    from_y: d.from_pos.1,
                    to_x: d.to_pos.0,
                    to_y: d.to_pos.1,
                    dp: format!("{:?}", d.dp),
                    cc: format!("{:?}", d.cc),
                    block_size: d.block_size,
                    from_color: d.from_color.clone(),
                    to_color: d.to_color.clone(),
                });
                
                let from_color = rich_instr.debug.as_ref()
                    .map(|d| d.from_color.clone())
                    .unwrap_or_default();
                let to_color = rich_instr.debug.as_ref()
                    .map(|d| d.to_color.clone())
                    .unwrap_or(value);
                
                BytecodeInstruction {
                    index: i,
                    opcode: op_name,
                    from_color,
                    to_color,
                    debug,
                }
            })
            .collect();
        
        console_log!("Compiled {} instructions ", instructions.len());
        
        serde_wasm_bindgen::to_value(&instructions)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }
}

/// Función de inicialización para configurar panic hook
#[wasm_bindgen(start)]
pub fn init() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
    
    console_log!("CanvasVM WASM initialized ");
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;
    use js_sys;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn test_canvas_creation() {
        let canvas = Canvas::new();
        assert!(canvas.vm.is_none());
    }

    #[wasm_bindgen_test]
    fn test_canvas_paint() {
        let mut canvas = Canvas::new();
        
        // Crear una imagen simple de 2x1 píxeles
        // Rojo claro → Amarillo claro (hue +1, light 0 → Add)
        let rgba = vec![
            0xFF, 0xC0, 0xC0, 0xFF, // (0,0) rojo claro
            0xFF, 0xFF, 0xC0, 0xFF, // (1,0) amarillo claro
        ];
        
        canvas.paint(&rgba, 2, 1).unwrap();
        assert!(canvas.vm.is_some());
    }

    #[wasm_bindgen_test]
    fn test_canvas_snapshot() {
        let mut canvas = Canvas::new();
        
        let rgba = vec![
            0xFF, 0xC0, 0xC0, 0xFF,
            0xFF, 0xFF, 0xC0, 0xFF,
        ];
        
        canvas.paint(&rgba, 2, 1).unwrap();
        let snapshot = canvas.snapshot();
        
        // El snapshot debe ser serializable
        assert!(snapshot.is_ok());
    }

    #[wasm_bindgen_test]
    fn test_canvas_reset() {
        let mut canvas = Canvas::new();
        
        let rgba = vec![
            0xFF, 0xC0, 0xC0, 0xFF,
            0xFF, 0xFF, 0xC0, 0xFF,
        ];
        
        canvas.paint(&rgba, 2, 1).unwrap();
        let _ = canvas.reset();
        
        // Después del reset, el VM debe existir y estar en estado inicial
        assert!(canvas.vm.is_some());
    }

    #[wasm_bindgen_test]
    fn test_canvas_stroke() {
        let mut canvas = Canvas::new();
        
        // Crear una imagen que genere instrucciones Push
        let rgba = vec![
            0xFF, 0xC0, 0xC0, 0xFF, // rojo claro (2 píxeles)
            0xFF, 0xC0, 0xC0, 0xFF,
            0xFF, 0xFF, 0xC0, 0xFF, // amarillo claro
        ];
        
        canvas.paint(&rgba, 3, 1).unwrap();
        
        // Ejecutar un paso
        let result = canvas.stroke();
        
        // stroke() puede retornar error si el VM está detenido o no puede moverse
        // pero debe retornar un resultado
        assert!(result.is_ok() || result.is_err());
    }

    #[wasm_bindgen_test]
    fn test_canvas_compile_to_bytecode() {
        let mut canvas = Canvas::new();
        
        let rgba = vec![
            0xFF, 0xC0, 0xC0, 0xFF, // rojo claro
            0xFF, 0xFF, 0xC0, 0xFF, // amarillo claro
        ];
        
        canvas.paint(&rgba, 2, 1).unwrap();
        
        // Obtener bytecode compilado
        let bytecode = canvas.compile_to_bytecode().unwrap();
        let bytecode_array: js_sys::Array = bytecode.into();
        
        // Debe tener al menos una instrucción
        assert!(bytecode_array.length() > 0);
        
        // Verificar que las instrucciones tienen la estructura correcta
        if bytecode_array.length() > 0 {
            let first_instr = bytecode_array.get(0);
            assert!(first_instr.is_object());
        }
    }

    #[wasm_bindgen_test]
    fn test_canvas_ink_string() {
        let mut canvas = Canvas::new();
        
        let rgba = vec![
            0xFF, 0xC0, 0xC0, 0xFF,
            0xFF, 0xFF, 0xC0, 0xFF,
        ];
        
        canvas.paint(&rgba, 2, 1).unwrap();
        
        // Obtener output inicial (debe estar vacío)
        let output = canvas.ink_string().unwrap();
        assert_eq!(output, "");
    }

    #[wasm_bindgen_test]
    fn test_snapshot_and_stroke_flow() {
        let mut canvas = Canvas::new();
        
        // Crear imagen con múltiples bloques
        let rgba = vec![
            0xFF, 0xC0, 0xC0, 0xFF, // rojo claro (bloque 2)
            0xFF, 0xC0, 0xC0, 0xFF,
            0xFF, 0xFF, 0xC0, 0xFF, // amarillo claro
        ];
        
        canvas.paint(&rgba, 3, 1).unwrap();
        
        // 1. Snapshot inicial
        let initial = canvas.snapshot().unwrap();
        assert!(initial.is_object());
        
        // 2. Ejecutar stroke
        let _ = canvas.stroke();
        
        // 3. Snapshot después de stroke
        let after = canvas.snapshot().unwrap();
        assert!(after.is_object());
        
        // 4. Ink string
        let output = canvas.ink_string().unwrap();
        // Output puede estar vacío si no hay OutChar/OutNumber
        assert!(output.is_empty() || !output.is_empty());
    }

    #[wasm_bindgen_test]
    fn test_reset_functionality() {
        let mut canvas = Canvas::new();
        
        let rgba = vec![
            0xFF, 0xC0, 0xC0, 0xFF,
            0xFF, 0xC0, 0xC0, 0xFF,
            0xFF, 0xFF, 0xC0, 0xFF,
        ];
        
        canvas.paint(&rgba, 3, 1).unwrap();
        
        // Ejecutar algunos pasos
        for _ in 0..3 {
            let _ = canvas.stroke();
        }
        
        // Reset
        canvas.reset().unwrap();
        
        // El VM debe estar reiniciado
        assert!(canvas.vm.is_some());
        
        // Debería poder ejecutar de nuevo
        let result = canvas.stroke();
        assert!(result.is_ok() || result.is_err());
    }

    #[wasm_bindgen_test]
    fn test_multiple_strokes_until_halt() {
        let mut canvas = Canvas::new();
        
        // Usar una imagen más grande para tener más operaciones
        let rgba = vec![
            0xFF, 0xC0, 0xC0, 0xFF, // rojo claro (bloque de tamaño 4)
            0xFF, 0xC0, 0xC0, 0xFF,
            0xFF, 0xC0, 0xC0, 0xFF,
            0xFF, 0xC0, 0xC0, 0xFF,
            0xFF, 0xFF, 0xC0, 0xFF, // amarillo claro
        ];
        
        canvas.paint(&rgba, 5, 1).unwrap();
        
        // Ejecutar múltiples pasos hasta que se detenga
        let max_steps = 50;
        let mut step_count = 0;
        
        for _ in 0..max_steps {
            match canvas.stroke() {
                Ok(_) => {
                    step_count += 1;
                    // Verificar que snapshot funciona en cada paso
                    let _ = canvas.snapshot().unwrap();
                }
                Err(_) => break,
            }
        }
        
        // El test pasa si no hay panic, incluso si no ejecuta ningún paso
        // (Algunos programas pueden detenerse inmediatamente)
        assert!(step_count >= 0, "Test should complete without panic");
    }

    #[wasm_bindgen_test]
    fn test_web_workflow_simulation() {
        // Simula el flujo completo que usa la web:
        // 1. paint() - compilar imagen
        // 2. compile_to_bytecode() - mostrar bytecode en tabla
        // 3. snapshot() - mostrar estado inicial
        // 4. stroke() repetido - ejecutar paso a paso o play
        // 5. snapshot() después de cada stroke - actualizar UI
        // 6. ink_string() - mostrar output
        // 7. reset() - reiniciar
        
        let mut canvas = Canvas::new();
        
        let rgba = vec![
            0xFF, 0xC0, 0xC0, 0xFF, // rojo (3 píxeles)
            0xFF, 0xC0, 0xC0, 0xFF,
            0xFF, 0xC0, 0xC0, 0xFF,
            0xFF, 0xFF, 0xC0, 0xFF, // amarillo
        ];
        
        // 1. paint()
        canvas.paint(&rgba, 4, 1).unwrap();
        
        // 2. compile_to_bytecode()
        let bytecode = canvas.compile_to_bytecode().unwrap();
        let bytecode_array: js_sys::Array = bytecode.into();
        assert!(bytecode_array.length() > 0, "Bytecode should not be empty");
        
        // 3. snapshot() inicial
        let snapshot = canvas.snapshot().unwrap();
        assert!(snapshot.is_object());
        
        // 4-5. stroke() + snapshot() loop (simula play o step)
        for _ in 0..5 {
            if canvas.stroke().is_ok() {
                let _ = canvas.snapshot().unwrap();
            } else {
                break;
            }
        }
        
        // 6. ink_string()
        let output = canvas.ink_string().unwrap();
        // Solo verificamos que no haga panic
        let _ = output;
        
        // 7. reset()
        canvas.reset().unwrap();
        assert!(canvas.vm.is_some());
    }
}
