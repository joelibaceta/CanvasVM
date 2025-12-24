/// Intermediate bytecode for optimized Piet execution
use crate::exits::{CodelChooser, Direction};
use serde::{Deserialize, Serialize};

/// Metadata about the compiled program
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgramMetadata {
    /// Codel size used during compilation (pixels per codel)
    pub codel_size: usize,
    /// Original image width in pixels
    pub image_width: usize,
    /// Original image height in pixels
    pub image_height: usize,
    /// Logical grid width in codels
    pub grid_width: usize,
    /// Logical grid height in codels
    pub grid_height: usize,
}

impl Default for ProgramMetadata {
    fn default() -> Self {
        Self {
            codel_size: 1,
            image_width: 0,
            image_height: 0,
            grid_width: 0,
            grid_height: 0,
        }
    }
}

/// Debug information for an instruction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstructionDebugInfo {
    /// Position of the source codel (x, y)
    pub from_pos: (usize, usize),
    /// Position of the destination codel (x, y)
    pub to_pos: (usize, usize),
    /// Direction Pointer when entering this instruction
    pub dp: Direction,
    /// Codel Chooser when entering this instruction
    pub cc: CodelChooser,
    /// Size of the source block (used for Push)
    pub block_size: usize,
    /// Color name of the source block
    pub from_color: String,
    /// Color name of the destination block
    pub to_color: String,
}

impl Default for InstructionDebugInfo {
    fn default() -> Self {
        Self {
            from_pos: (0, 0),
            to_pos: (0, 0),
            dp: Direction::Right,
            cc: CodelChooser::Left,
            block_size: 1,
            from_color: String::new(),
            to_color: String::new(),
        }
    }
}

/// Rich instruction with debug information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RichInstruction {
    /// The actual operation
    pub op: Instruction,
    /// Debug information (optional, can be stripped for production)
    pub debug: Option<InstructionDebugInfo>,
}

impl RichInstruction {
    /// Creates a new rich instruction with debug info
    pub fn new(op: Instruction, debug: InstructionDebugInfo) -> Self {
        Self { op, debug: Some(debug) }
    }
    
    /// Creates a rich instruction without debug info
    pub fn simple(op: Instruction) -> Self {
        Self { op, debug: None }
    }
}

/// Bytecode instruction
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Instruction {
    /// Push: pushes block size to stack
    Push(i32),
    /// Pop: removes top of stack
    Pop,
    /// Add: adds two values from top
    Add,
    /// Subtract: subtracts (second - first)
    Subtract,
    /// Multiply: multiplies two values from top
    Multiply,
    /// Divide: divides (second / first)
    Divide,
    /// Mod: modulo (second % first)
    Mod,
    /// Not: logical negation (0 → 1, other → 0)
    Not,
    /// Greater: compares (second > first) → 1 or 0
    Greater,
    /// Pointer: rotates DP (n times clockwise)
    Pointer,
    /// Switch: changes CC if n is odd
    Switch,
    /// Duplicate: duplicates top of stack
    Duplicate,
    /// Roll: rolls depth elements, times times
    Roll,
    /// InNumber: reads a number from input
    InNumber,
    /// InChar: reads a character from input
    InChar,
    /// OutNumber: writes a number to output
    OutNumber,
    /// OutChar: writes a character to output
    OutChar,
    /// Nop: does nothing (white→white transition or no hue/lightness change)
    Nop,
    /// Halt: halts the VM
    Halt,
}

/// Compiled program (bytecode + metadata)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Program {
    /// Program metadata (codel size, dimensions, etc.)
    pub metadata: ProgramMetadata,
    /// Rich instructions with debug information
    pub rich_instructions: Vec<RichInstruction>,
    /// Simple instructions (legacy, for VM execution)
    pub instructions: Vec<Instruction>,
    /// Mapping from image position to instruction index
    /// Allows knowing which instruction would execute from each position
    pub position_map: Vec<Vec<Option<usize>>>, // [y][x] -> instruction_index
    /// Mapping from position to next position after execution
    /// This allows the VM to know where to move
    pub next_position: Vec<Vec<Option<(usize, usize)>>>, // [y][x] -> (next_x, next_y)
    /// Original grid width (in codels)
    pub width: usize,
    /// Original grid height (in codels)
    pub height: usize,
}

impl Program {
    /// Creates an empty program
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            metadata: ProgramMetadata {
                codel_size: 1,
                image_width: width,
                image_height: height,
                grid_width: width,
                grid_height: height,
            },
            rich_instructions: Vec::new(),
            instructions: Vec::new(),
            position_map: vec![vec![None; width]; height],
            next_position: vec![vec![None; width]; height],
            width,
            height,
        }
    }
    
    /// Creates a program with explicit metadata
    pub fn with_metadata(metadata: ProgramMetadata) -> Self {
        let width = metadata.grid_width;
        let height = metadata.grid_height;
        Self {
            metadata,
            rich_instructions: Vec::new(),
            instructions: Vec::new(),
            position_map: vec![vec![None; width]; height],
            next_position: vec![vec![None; width]; height],
            width,
            height,
        }
    }

    /// Adds an instruction and returns its index
    pub fn add_instruction(&mut self, instr: Instruction) -> usize {
        let idx = self.instructions.len();
        self.instructions.push(instr.clone());
        self.rich_instructions.push(RichInstruction::simple(instr));
        idx
    }
    
    /// Adds a rich instruction with debug info and returns its index
    pub fn add_rich_instruction(&mut self, instr: Instruction, debug: InstructionDebugInfo) -> usize {
        let idx = self.instructions.len();
        self.instructions.push(instr.clone());
        self.rich_instructions.push(RichInstruction::new(instr, debug));
        idx
    }

    /// Associates a position with an instruction
    pub fn map_position(&mut self, x: usize, y: usize, instr_idx: usize) {
        if y < self.height && x < self.width {
            self.position_map[y][x] = Some(instr_idx);
        }
    }

    /// Associates a position with its next position
    pub fn map_next_position(&mut self, x: usize, y: usize, next_x: usize, next_y: usize) {
        if y < self.height && x < self.width {
            self.next_position[y][x] = Some((next_x, next_y));
        }
    }

    /// Gets the next position from current position
    pub fn get_next_position(&self, x: usize, y: usize) -> Option<(usize, usize)> {
        if y < self.height && x < self.width {
            self.next_position[y][x]
        } else {
            None
        }
    }

    /// Gets the instruction at a position
    pub fn get_instruction_at(&self, x: usize, y: usize) -> Option<&Instruction> {
        if y < self.height && x < self.width {
            if let Some(idx) = self.position_map[y][x] {
                return self.instructions.get(idx);
            }
        }
        None
    }
    
    /// Gets the rich instruction at a position
    pub fn get_rich_instruction_at(&self, x: usize, y: usize) -> Option<&RichInstruction> {
        if y < self.height && x < self.width {
            if let Some(idx) = self.position_map[y][x] {
                return self.rich_instructions.get(idx);
            }
        }
        None
    }
    
    /// Gets the instruction index at a position
    pub fn get_instruction_index_at(&self, x: usize, y: usize) -> Option<usize> {
        if y < self.height && x < self.width {
            self.position_map[y][x]
        } else {
            None
        }
    }
    
    /// Gets a rich instruction by index
    pub fn get_rich_instruction(&self, idx: usize) -> Option<&RichInstruction> {
        self.rich_instructions.get(idx)
    }

    /// Returns the total number of instructions
    pub fn len(&self) -> usize {
        self.instructions.len()
    }

    /// Verifica si el programa está vacío
    pub fn is_empty(&self) -> bool {
        self.instructions.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_program_creation() {
        let mut program = Program::new(10, 10);
        
        let idx = program.add_instruction(Instruction::Push(5));
        assert_eq!(idx, 0);
        
        program.map_position(3, 4, idx);
        assert_eq!(program.get_instruction_at(3, 4), Some(&Instruction::Push(5)));
    }

    #[test]
    fn test_instruction_serialization() {
        let instr = Instruction::Add;
        let json = serde_json::to_string(&instr).unwrap();
        let deserialized: Instruction = serde_json::from_str(&json).unwrap();
        assert_eq!(instr, deserialized);
    }
}
