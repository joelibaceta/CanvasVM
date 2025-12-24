//! Debugger for step-by-step execution with rich state information
//!
//! This module provides debugging capabilities for the Piet VM, allowing
//! step-by-step execution with full state inspection.

use crate::bytecode::{Instruction, InstructionDebugInfo, Program, RichInstruction};
use crate::compiler::{CompileMode, Compiler};
use crate::error::VmError;
use crate::exits::{CodelChooser, Direction, Position};
use crate::grid::Grid;
use crate::io::{Input, Output};
use serde::{Deserialize, Serialize};

/// Execution mode for the debugger
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionMode {
    /// Run mode - execute without debug overhead
    Run,
    /// Debug mode - step-by-step with full state
    Debug,
    /// Run with debug info - execute all but keep trace
    RunWithTrace,
}

/// A single step in the execution trace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionStep {
    /// Step number (0-indexed)
    pub step: usize,
    /// The instruction executed
    pub instruction: Instruction,
    /// Debug info for this instruction (if compiled in debug mode)
    pub debug_info: Option<InstructionDebugInfo>,
    /// Stack before execution
    pub stack_before: Vec<i32>,
    /// Stack after execution
    pub stack_after: Vec<i32>,
    /// Position before this step (in codels)
    pub position: (usize, usize),
    /// Direction pointer
    pub dp: Direction,
    /// Codel chooser
    pub cc: CodelChooser,
    /// Output produced by this step (if any)
    pub output: Option<i32>,
    /// Whether this was an output character
    pub output_is_char: bool,
}

/// Full execution trace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionTrace {
    /// All steps executed
    pub steps: Vec<ExecutionStep>,
    /// Final output (numbers)
    pub output: Vec<i32>,
    /// Final output as string
    pub output_string: String,
    /// Total steps executed
    pub total_steps: usize,
    /// Whether execution completed (halted normally)
    pub completed: bool,
    /// Error if execution failed
    pub error: Option<String>,
}

/// Current debugger state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebuggerState {
    /// Current instruction pointer
    pub ip: usize,
    /// Current position in the grid (codels)
    pub position: (usize, usize),
    /// Current position in pixels (for highlighting)
    pub pixel_position: (usize, usize),
    /// Direction pointer
    pub dp: Direction,
    /// Codel chooser  
    pub cc: CodelChooser,
    /// Current stack
    pub stack: Vec<i32>,
    /// Whether VM is halted
    pub halted: bool,
    /// Whether VM is waiting for input
    pub waiting_for_input: Option<InputRequest>,
    /// Total steps executed
    pub steps: usize,
    /// Current instruction (if any)
    pub current_instruction: Option<RichInstruction>,
    /// Next instruction (if any)
    pub next_instruction: Option<RichInstruction>,
    /// Output so far
    pub output: Vec<i32>,
    /// Output as string
    pub output_string: String,
}

/// Type of input being requested
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InputRequest {
    /// Waiting for a number input (InNumber instruction)
    Number,
    /// Waiting for a character input (InChar instruction)
    Char,
}

/// Debugger for Piet programs
/// 
/// Provides step-by-step execution with full state inspection.
pub struct Debugger {
    /// Compiled program with debug info
    program: Program,
    /// The grid (for visualization)
    grid: Grid,
    /// Codel size
    codel_size: usize,
    /// Current instruction pointer
    ip: usize,
    /// Current position in the grid
    position: Position,
    /// Direction pointer
    dp: Direction,
    /// Codel chooser
    cc: CodelChooser,
    /// Value stack
    stack: Vec<i32>,
    /// Input buffer
    input: Input,
    /// Output buffer
    output: Output,
    /// Is halted?
    halted: bool,
    /// Is waiting for input?
    waiting_for_input: Option<InputRequest>,
    /// Steps executed
    steps: usize,
    /// Execution trace (if recording)
    trace: Vec<ExecutionStep>,
    /// Whether to record trace
    record_trace: bool,
    /// Breakpoints (instruction indices)
    breakpoints: Vec<usize>,
}

impl Debugger {
    /// Create a new debugger from a grid
    pub fn new(grid: Grid, codel_size: usize, image_width: usize, image_height: usize) -> Result<Self, VmError> {
        // Always compile in debug mode for the debugger
        let compiler = Compiler::with_codel_size(grid.clone(), codel_size, image_width, image_height)
            .with_mode(CompileMode::Debug);
        let program = compiler.compile()?;
        
        Ok(Self {
            program,
            grid,
            codel_size,
            ip: 0,
            position: Position::new(0, 0),
            dp: Direction::Right,
            cc: CodelChooser::Left,
            stack: Vec::new(),
            input: Input::new(),
            output: Output::new(),
            halted: false,
            waiting_for_input: None,
            steps: 0,
            trace: Vec::new(),
            record_trace: true,
            breakpoints: Vec::new(),
        })
    }

    /// Create debugger from an existing program
    pub fn from_program(program: Program, grid: Grid, codel_size: usize) -> Self {
        Self {
            program,
            grid,
            codel_size,
            ip: 0,
            position: Position::new(0, 0),
            dp: Direction::Right,
            cc: CodelChooser::Left,
            stack: Vec::new(),
            input: Input::new(),
            output: Output::new(),
            halted: false,
            waiting_for_input: None,
            steps: 0,
            trace: Vec::new(),
            record_trace: true,
            breakpoints: Vec::new(),
        }
    }

    /// Enable or disable trace recording
    pub fn set_record_trace(&mut self, record: bool) {
        self.record_trace = record;
    }

    /// Add a breakpoint at instruction index
    pub fn add_breakpoint(&mut self, index: usize) {
        if !self.breakpoints.contains(&index) {
            self.breakpoints.push(index);
        }
    }

    /// Remove a breakpoint
    pub fn remove_breakpoint(&mut self, index: usize) {
        self.breakpoints.retain(|&bp| bp != index);
    }

    /// Clear all breakpoints
    pub fn clear_breakpoints(&mut self) {
        self.breakpoints.clear();
    }

    /// Get all breakpoints
    pub fn breakpoints(&self) -> &[usize] {
        &self.breakpoints
    }

    /// Reset the debugger to initial state
    pub fn reset(&mut self) {
        self.ip = 0;
        self.position = Position::new(0, 0);
        self.dp = Direction::Right;
        self.cc = CodelChooser::Left;
        self.stack.clear();
        self.input = Input::new();
        self.output = Output::new();
        self.halted = false;
        self.waiting_for_input = None;
        self.steps = 0;
        self.trace.clear();
    }

    /// Get current state
    pub fn state(&self) -> DebuggerState {
        let current = if self.ip < self.program.rich_instructions.len() {
            Some(self.program.rich_instructions[self.ip].clone())
        } else {
            None
        };
        
        let next = if self.ip + 1 < self.program.rich_instructions.len() {
            Some(self.program.rich_instructions[self.ip + 1].clone())
        } else {
            None
        };

        // Calculate pixel position from codel position
        let pixel_x = self.position.x * self.codel_size;
        let pixel_y = self.position.y * self.codel_size;

        DebuggerState {
            ip: self.ip,
            position: (self.position.x, self.position.y),
            pixel_position: (pixel_x, pixel_y),
            dp: self.dp,
            cc: self.cc,
            stack: self.stack.clone(),
            halted: self.halted,
            waiting_for_input: self.waiting_for_input.clone(),
            steps: self.steps,
            current_instruction: current,
            next_instruction: next,
            output: self.output.read(),
            output_string: self.output.read_string(),
        }
    }

    /// Get the program
    pub fn program(&self) -> &Program {
        &self.program
    }

    /// Get execution trace
    pub fn trace(&self) -> &[ExecutionStep] {
        &self.trace
    }

    /// Get full trace as ExecutionTrace
    pub fn get_execution_trace(&self) -> ExecutionTrace {
        ExecutionTrace {
            steps: self.trace.clone(),
            output: self.output.read(),
            output_string: self.output.read_string(),
            total_steps: self.steps,
            completed: self.halted,
            error: None,
        }
    }

    /// Provide input number
    pub fn input(&mut self, value: i32) {
        self.input.write(value);
        // Clear waiting state if we were waiting for input
        if self.waiting_for_input.is_some() {
            self.waiting_for_input = None;
        }
    }

    /// Provide input character
    pub fn input_char(&mut self, c: char) {
        self.input.write_char(c);
        // Clear waiting state if we were waiting for input
        if self.waiting_for_input.is_some() {
            self.waiting_for_input = None;
        }
    }

    /// Provide input and resume execution (for interactive mode)
    /// This is the main method to call from the UI when the user provides input
    pub fn provide_input(&mut self, value: i32) {
        self.input.write(value);
        self.waiting_for_input = None;
    }

    /// Provide character input and resume execution (for interactive mode)
    pub fn provide_input_char(&mut self, c: char) {
        self.input.write_char(c);
        self.waiting_for_input = None;
    }

    /// Check if waiting for input
    pub fn is_waiting_for_input(&self) -> bool {
        self.waiting_for_input.is_some()
    }

    /// Get the type of input being waited for
    pub fn get_input_request(&self) -> Option<InputRequest> {
        self.waiting_for_input.clone()
    }

    /// Load text as character inputs (each character becomes an input for in_char)
    pub fn load_input_text(&mut self, text: &str) {
        self.input.load_text(text);
    }

    /// Load numbers from string (whitespace-separated, for in_number operations)
    pub fn load_input_numbers(&mut self, text: &str) {
        self.input.load_numbers(text);
    }

    /// Load a vector of numbers as inputs
    pub fn load_input_number_vec(&mut self, numbers: &[i32]) {
        self.input.load_number_vec(numbers);
    }

    /// Clear all inputs
    pub fn clear_input(&mut self) {
        self.input.clear();
    }

    /// Rewind input buffer to start
    pub fn rewind_input(&mut self) {
        self.input.rewind();
    }

    /// Check if there are inputs available
    pub fn has_input(&self) -> bool {
        self.input.has_input()
    }

    /// Get remaining input count
    pub fn remaining_input(&self) -> usize {
        self.input.remaining()
    }

    /// Execute a single step
    pub fn step(&mut self) -> Result<Option<ExecutionStep>, VmError> {
        if self.halted {
            return Err(VmError::Halted);
        }

        // If waiting for input, don't proceed until input is provided
        if self.waiting_for_input.is_some() {
            return Ok(None);
        }

        if self.ip >= self.program.rich_instructions.len() {
            self.halted = true;
            return Err(VmError::Halted);
        }

        let rich_instr = &self.program.rich_instructions[self.ip];
        let instruction = rich_instr.op.clone();
        let debug_info = rich_instr.debug.clone();

        // Check if this instruction needs input and we don't have any
        match &instruction {
            Instruction::InNumber => {
                if !self.input.has_input() {
                    self.waiting_for_input = Some(InputRequest::Number);
                    return Ok(None);
                }
            }
            Instruction::InChar => {
                if !self.input.has_input() {
                    self.waiting_for_input = Some(InputRequest::Char);
                    return Ok(None);
                }
            }
            _ => {}
        }

        // Capture state before execution
        let stack_before = self.stack.clone();
        let position_before = (self.position.x, self.position.y);
        let dp_before = self.dp;
        let cc_before = self.cc;

        // Update position from debug info if available
        if let Some(ref info) = debug_info {
            self.position = Position::new(info.from_pos.0, info.from_pos.1);
            self.dp = info.dp;
            self.cc = info.cc;
        }

        // Execute the instruction
        let mut step_output: Option<i32> = None;
        let mut output_is_char = false;

        match &instruction {
            Instruction::Push(n) => {
                self.stack.push(*n);
            }
            Instruction::Pop => {
                self.stack.pop();
            }
            Instruction::Add => {
                if self.stack.len() >= 2 {
                    let a = self.stack.pop().unwrap();
                    let b = self.stack.pop().unwrap();
                    self.stack.push(b.wrapping_add(a));
                }
            }
            Instruction::Subtract => {
                if self.stack.len() >= 2 {
                    let a = self.stack.pop().unwrap();
                    let b = self.stack.pop().unwrap();
                    self.stack.push(b.wrapping_sub(a));
                }
            }
            Instruction::Multiply => {
                if self.stack.len() >= 2 {
                    let a = self.stack.pop().unwrap();
                    let b = self.stack.pop().unwrap();
                    self.stack.push(b.wrapping_mul(a));
                }
            }
            Instruction::Divide => {
                if self.stack.len() >= 2 {
                    let a = self.stack.pop().unwrap();
                    let b = self.stack.pop().unwrap();
                    if a != 0 {
                        self.stack.push(b / a);
                    } else {
                        // Division by zero - push back values
                        self.stack.push(b);
                        self.stack.push(a);
                    }
                }
            }
            Instruction::Mod => {
                if self.stack.len() >= 2 {
                    let a = self.stack.pop().unwrap();
                    let b = self.stack.pop().unwrap();
                    if a != 0 {
                        self.stack.push(b % a);
                    } else {
                        self.stack.push(b);
                        self.stack.push(a);
                    }
                }
            }
            Instruction::Not => {
                if let Some(a) = self.stack.pop() {
                    self.stack.push(if a == 0 { 1 } else { 0 });
                }
            }
            Instruction::Greater => {
                if self.stack.len() >= 2 {
                    let a = self.stack.pop().unwrap();
                    let b = self.stack.pop().unwrap();
                    self.stack.push(if b > a { 1 } else { 0 });
                }
            }
            Instruction::Pointer => {
                if let Some(n) = self.stack.pop() {
                    let rotations = n.rem_euclid(4);
                    self.dp = self.dp.rotate_clockwise(rotations);
                }
            }
            Instruction::Switch => {
                if let Some(n) = self.stack.pop() {
                    if n % 2 != 0 {
                        self.cc = self.cc.toggle();
                    }
                }
            }
            Instruction::Duplicate => {
                if let Some(&top) = self.stack.last() {
                    self.stack.push(top);
                }
            }
            Instruction::Roll => {
                if self.stack.len() >= 2 {
                    let times = self.stack.pop().unwrap();
                    let depth = self.stack.pop().unwrap();
                    
                    if depth > 0 && (depth as usize) <= self.stack.len() {
                        let depth = depth as usize;
                        let len = self.stack.len();
                        let start = len - depth;
                        let times = times.rem_euclid(depth as i32) as usize;
                        
                        if times > 0 {
                            let mut temp: Vec<i32> = self.stack.drain(start..).collect();
                            temp.rotate_right(times);
                            self.stack.extend(temp);
                        }
                    }
                }
            }
            Instruction::InNumber => {
                if let Some(n) = self.input.read() {
                    self.stack.push(n);
                }
            }
            Instruction::InChar => {
                if let Some(c) = self.input.read() {
                    self.stack.push(c);
                }
            }
            Instruction::OutNumber => {
                if let Some(n) = self.stack.pop() {
                    self.output.write_number(n);
                    step_output = Some(n);
                }
            }
            Instruction::OutChar => {
                if let Some(n) = self.stack.pop() {
                    self.output.write_char(n);
                    step_output = Some(n);
                    output_is_char = true;
                }
            }
            Instruction::Nop => {
                // Do nothing
            }
            Instruction::Halt => {
                self.halted = true;
            }
        }

        // Update position to destination if available
        if let Some(ref info) = debug_info {
            self.position = Position::new(info.to_pos.0, info.to_pos.1);
        }

        self.ip += 1;
        self.steps += 1;

        // Create execution step
        let exec_step = ExecutionStep {
            step: self.steps - 1,
            instruction,
            debug_info,
            stack_before,
            stack_after: self.stack.clone(),
            position: position_before,
            dp: dp_before,
            cc: cc_before,
            output: step_output,
            output_is_char,
        };

        // Record if tracing
        if self.record_trace {
            self.trace.push(exec_step.clone());
        }

        Ok(Some(exec_step))
    }

    /// Run until halt or breakpoint
    pub fn run(&mut self) -> Result<ExecutionTrace, VmError> {
        while !self.halted {
            // Check for breakpoint before executing
            if self.breakpoints.contains(&self.ip) && self.steps > 0 {
                break;
            }
            self.step()?;
        }
        Ok(self.get_execution_trace())
    }

    /// Run for a maximum number of steps
    pub fn run_steps(&mut self, max_steps: usize) -> Result<usize, VmError> {
        let mut executed = 0;
        while !self.halted && executed < max_steps {
            if self.breakpoints.contains(&self.ip) && executed > 0 {
                break;
            }
            self.step()?;
            executed += 1;
        }
        Ok(executed)
    }

    /// Run with a step limit, returning execution trace
    /// Similar to run() but stops after max_steps with a timeout error in the trace
    pub fn run_limited(&mut self, max_steps: usize) -> Result<ExecutionTrace, VmError> {
        let mut executed = 0;
        while !self.halted && executed < max_steps {
            // Check for breakpoint before executing
            if self.breakpoints.contains(&self.ip) && self.steps > 0 {
                break;
            }
            self.step()?;
            executed += 1;
        }
        
        let mut trace = self.get_execution_trace();
        
        // If we hit the limit without halting, mark it as a timeout
        if !self.halted && executed >= max_steps {
            trace.error = Some(format!("Watchdog timeout: execution limit of {} steps reached", max_steps));
            trace.completed = false;
        }
        
        Ok(trace)
    }

    /// Run until next breakpoint
    pub fn continue_to_breakpoint(&mut self) -> Result<Option<usize>, VmError> {
        while !self.halted {
            self.step()?;
            if self.breakpoints.contains(&self.ip) {
                return Ok(Some(self.ip));
            }
        }
        Ok(None)
    }

    /// Get instruction at index
    pub fn get_instruction(&self, index: usize) -> Option<&RichInstruction> {
        self.program.rich_instructions.get(index)
    }

    /// Get total instruction count
    pub fn instruction_count(&self) -> usize {
        self.program.rich_instructions.len()
    }

    /// Is at a breakpoint?
    pub fn is_at_breakpoint(&self) -> bool {
        self.breakpoints.contains(&self.ip)
    }

    /// Get current instruction index
    pub fn current_ip(&self) -> usize {
        self.ip
    }

    /// Is halted?
    pub fn is_halted(&self) -> bool {
        self.halted
    }

    /// Get output
    pub fn output(&self) -> &Output {
        &self.output
    }

    /// Get output as string
    pub fn output_string(&self) -> String {
        self.output.read_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_grid() -> Grid {
        // Create a simple 3x1 grid: LightRed(2 codels) -> LightYellow
        // This should generate Push(2) instruction
        let rgba = vec![
            0xFF, 0xC0, 0xC0, 0xFF, // LightRed
            0xFF, 0xC0, 0xC0, 0xFF, // LightRed
            0xFF, 0xFF, 0xC0, 0xFF, // LightYellow
        ];
        Grid::from_rgba(3, 1, &rgba).unwrap()
    }

    #[test]
    fn test_debugger_creation() {
        let grid = create_test_grid();
        let debugger = Debugger::new(grid, 1, 3, 1).unwrap();
        
        assert!(!debugger.is_halted());
        assert_eq!(debugger.current_ip(), 0);
    }

    #[test]
    fn test_debugger_step() {
        let grid = create_test_grid();
        let mut debugger = Debugger::new(grid, 1, 3, 1).unwrap();
        
        let state_before = debugger.state();
        assert_eq!(state_before.ip, 0);
        assert!(state_before.stack.is_empty());
        
        // Execute one step
        let step = debugger.step().unwrap();
        assert!(step.is_some());
        
        let state_after = debugger.state();
        assert_eq!(state_after.ip, 1);
    }

    #[test]
    fn test_debugger_reset() {
        let grid = create_test_grid();
        let mut debugger = Debugger::new(grid, 1, 3, 1).unwrap();
        
        // Run some steps
        let _ = debugger.run_steps(5);
        
        // Reset
        debugger.reset();
        
        assert_eq!(debugger.current_ip(), 0);
        assert!(!debugger.is_halted());
        assert!(debugger.state().stack.is_empty());
    }

    #[test]
    fn test_debugger_breakpoints() {
        let grid = create_test_grid();
        let mut debugger = Debugger::new(grid, 1, 3, 1).unwrap();
        
        debugger.add_breakpoint(1);
        assert!(debugger.breakpoints().contains(&1));
        
        debugger.remove_breakpoint(1);
        assert!(!debugger.breakpoints().contains(&1));
    }

    #[test]
    fn test_debugger_trace() {
        let grid = create_test_grid();
        let mut debugger = Debugger::new(grid, 1, 3, 1).unwrap();
        debugger.set_record_trace(true);
        
        // Run all
        let _ = debugger.run();
        
        let trace = debugger.get_execution_trace();
        assert!(trace.steps.len() > 0);
        assert!(trace.completed);
    }
}
