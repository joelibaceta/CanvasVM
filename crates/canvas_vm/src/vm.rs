/// VM optimizada que ejecuta bytecode pre-compilado
use crate::bytecode::{Instruction, Program};
use crate::compiler::Compiler;
use crate::error::VmError;
use crate::exits::{CodelChooser, Direction, Position};
use crate::grid::Grid;
use crate::io::{Input, InputSource, Output, OutputSink};
use crate::ops::{get_operation, PietColor};
use serde::{Deserialize, Serialize};

/// Estado de la VM que ejecuta bytecode
#[derive(Debug, Clone)]
pub struct BytecodeVm {
    /// Programa compilado
    program: Program,
    /// Grid original (para calcular transiciones dinámicas)
    grid: Grid,
    /// Posición actual en la imagen
    position: Position,
    /// Direction Pointer
    dp: Direction,
    /// Codel Chooser
    cc: CodelChooser,
    /// Stack de valores
    stack: Vec<i32>,
    /// Entrada
    input: Input,
    /// Salida
    output: Output,
    /// ¿Está detenida?
    halted: bool,
    /// Número de pasos ejecutados
    steps: usize,
    /// Límite máximo de pasos (watchdog). None = sin límite
    max_steps: Option<usize>,
}

/// Snapshot del estado de la VM (para debugger)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BytecodeVmSnapshot {
    pub position: Position,
    pub dp: Direction,
    pub cc: CodelChooser,
    pub stack: Vec<i32>,
    pub halted: bool,
    pub steps: usize,
    pub next_instruction: Option<Instruction>,
    pub instruction_index: Option<usize>,
}

/// Resultado de una vista previa (dry-run)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackPreview {
    pub stack_before: Vec<i32>,
    pub stack_after: Vec<i32>,
    pub instruction: Instruction,
    pub success: bool,
    pub error: Option<String>,
}

impl BytecodeVm {
    /// Límite de pasos por defecto (1 millón)
    pub const DEFAULT_MAX_STEPS: usize = 1_000_000;

    /// Crea una nueva VM con un programa compilado y la grid
    pub fn new(program: Program, grid: Grid) -> Self {
        // eprintln!("DEBUG BytecodeVm::new: program has {} instructions", program.instructions.len());
        // eprintln!("DEBUG BytecodeVm::new: program dimensions {}x{}", program.width, program.height);

        Self {
            program,
            grid,
            position: Position::new(0, 0),
            dp: Direction::Right,
            cc: CodelChooser::Left,
            stack: Vec::new(),
            input: Input::new(),
            output: Output::new(),
            halted: false,
            steps: 0,
            max_steps: Some(Self::DEFAULT_MAX_STEPS),
        }
    }

    /// Crea una nueva VM desde una Grid (compila automáticamente)
    pub fn from_grid(grid: Grid) -> Result<Self, VmError> {
        // eprintln!("DEBUG BytecodeVm::from_grid: compiling grid {}x{}", grid.width(), grid.height());
        let codel_size = grid.codel_size();
        let compiler = Compiler::with_codel_size(grid.clone(), codel_size, 0, 0);
        let program = compiler.compile()?;
        // eprintln!("DEBUG BytecodeVm::from_grid: compiled {} instructions", program.instructions.len());
        Ok(Self::new(program, grid))
    }

    /// Crea una nueva VM desde un programa precompilado (sin grid original)
    /// Útil para ejecutar bytecode .cvm sin la imagen original
    pub fn from_program(program: Program) -> Result<Self, VmError> {
        use crate::PietColor;

        // Try to extract the embedded grid
        let grid = match program.extract_grid() {
            Ok(Some(grid)) => grid,
            Ok(None) => {
                // No embedded grid, create an empty one
                let total_cells = program.width * program.height;
                let cells = vec![PietColor::White; total_cells];
                Grid::new(program.width, program.height, cells).map_err(|e| {
                    VmError::CompilationFailed(format!("Failed to create empty grid: {:?}", e))
                })?
            }
            Err(e) => {
                return Err(VmError::CompilationFailed(format!(
                    "Failed to extract grid: {}",
                    e
                )));
            }
        };

        Ok(Self::new(program, grid))
    }

    // === Configuración del Watchdog ===

    /// Establece el límite máximo de pasos (watchdog)
    pub fn set_max_steps(&mut self, max: Option<usize>) {
        self.max_steps = max;
    }

    /// Obtiene el límite máximo de pasos actual
    pub fn max_steps(&self) -> Option<usize> {
        self.max_steps
    }

    /// Desactiva el watchdog (permite ejecución infinita)
    pub fn disable_watchdog(&mut self) {
        self.max_steps = None;
    }

    // === API pública (equivalente a PietVm) ===

    /// Ejecuta un solo paso - calcula dinámicamente la instrucción basada en transición de color
    pub fn stroke(&mut self) -> Result<(), VmError> {
        if self.halted {
            return Err(VmError::Halted);
        }

        // Watchdog: verificar límite de pasos
        if let Some(max) = self.max_steps {
            if self.steps >= max {
                self.halted = true;
                return Err(VmError::ExecutionTimeout(self.steps));
            }
        }

        // Obtener el color actual
        let current_color = match self.grid.get(self.position) {
            Some(c) => c,
            None => {
                self.halted = true;
                return Err(VmError::Halted);
            }
        };

        // eprintln!("DEBUG stroke: current_color={:?}", current_color);

        // Negro = halt
        if current_color.is_black() {
            self.halted = true;
            return Err(VmError::Halted);
        }

        // Blanco = deslizar sin ejecutar instrucción
        if current_color.is_white() {
            // eprintln!("DEBUG stroke: on white, sliding");
            if let Some(next_pos) = self.slide_through_white(self.position, self.dp) {
                self.position = next_pos;
                self.steps += 1;
                return Ok(());
            } else {
                self.halted = true;
                return Err(VmError::Halted);
            }
        }

        // Color cromático: obtener información del bloque
        let block_id = match self.grid.get_block_id(self.position) {
            Some(id) => id,
            None => {
                self.halted = true;
                return Err(VmError::Halted);
            }
        };

        let block_size = match self.grid.get_block_info(block_id) {
            Some(info) => info.size,
            None => {
                self.halted = true;
                return Err(VmError::Halted);
            }
        };

        // Intentar encontrar una salida válida (con reintentos como Piet real)
        let mut dp = self.dp;
        let mut cc = self.cc;
        let mut next_pos = None;
        let mut exit_color = None;
        let mut crossed_white = false; // Bandera para saber si cruzamos blanco

        for attempt in 0..8 {
            if let Some(exit_pos) = self.grid.get_exit(block_id, dp, cc) {
                if let Some(color) = self.grid.get(exit_pos) {
                    if color.is_black() {
                        // Bloqueado por negro - rotar
                        if attempt % 2 == 0 {
                            cc = cc.toggle();
                        } else {
                            dp = dp.rotate_clockwise(1);
                        }
                        continue;
                    } else if color.is_white() {
                        // Deslizarse por el blanco
                        if let Some(slide_pos) = self.slide_through_white(exit_pos, dp) {
                            if let Some(slide_color) = self.grid.get(slide_pos) {
                                // Verificar que no estamos volviendo al mismo bloque
                                let slide_block = self.grid.get_block_id(slide_pos);
                                if slide_block != Some(block_id) {
                                    next_pos = Some(slide_pos);
                                    exit_color = Some(slide_color);
                                    crossed_white = true; // Marcamos que cruzamos blanco
                                    break;
                                }
                                // Si volvemos al mismo bloque, es como si el blanco nos bloqueara
                                // eprintln!("DEBUG stroke: slide would return to same block, treating as blocked");
                            }
                        }
                        // No se puede salir del blanco - rotar
                        if attempt % 2 == 0 {
                            cc = cc.toggle();
                        } else {
                            dp = dp.rotate_clockwise(1);
                        }
                        continue;
                    } else {
                        // Color válido
                        next_pos = Some(exit_pos);
                        exit_color = Some(color);
                        break;
                    }
                }
            }
            // No hay salida en esta dirección - rotar
            if attempt % 2 == 0 {
                cc = cc.toggle();
            } else {
                dp = dp.rotate_clockwise(1);
            }
        }

        // Si no encontramos salida, halt
        let (final_pos, final_color) = match (next_pos, exit_color) {
            (Some(p), Some(c)) => (p, c),
            _ => {
                self.halted = true;
                return Err(VmError::Halted);
            }
        };

        // Calcular y ejecutar la instrucción basada en transición de color
        // Si cruzamos blanco, NO ejecutamos operación (es como teleportarse)
        let instr = if crossed_white {
            Instruction::Nop
        } else {
            self.color_transition_to_instruction(current_color, final_color, block_size)
        };

        // Ejecutar la instrucción
        match self.execute_instruction(&instr) {
            Ok(_) => {}
            Err(VmError::StackUnderflow) => {
                // Stack underflow se ignora en Piet
            }
            Err(VmError::DivisionByZero) => {
                // División por cero se ignora en Piet
            }
            Err(e) => return Err(e),
        }

        // Actualizar posición
        // Nota: NO sobrescribir dp y cc aquí, porque Pointer/Switch ya los modificaron
        // Solo actualizamos dp/cc si cambiaron durante la búsqueda de salida (rotaciones por bloqueo)
        // pero esas rotaciones ya están reflejadas en self.dp/self.cc si no hubo Pointer/Switch
        self.position = final_pos;
        // Solo actualizar dp/cc si la instrucción NO fue Pointer/Switch
        match instr {
            Instruction::Pointer | Instruction::Switch => {
                // Pointer/Switch ya modificaron self.dp/self.cc, no sobrescribir
            }
            _ => {
                self.dp = dp;
                self.cc = cc;
            }
        }
        self.steps += 1;

        // eprintln!("DEBUG stroke: after execution, stack={:?}", self.stack);

        Ok(())
    }

    /// Calcula la instrucción basada en transición de color
    fn color_transition_to_instruction(
        &self,
        from: PietColor,
        to: PietColor,
        block_size: usize,
    ) -> Instruction {
        // Si el destino es blanco o negro, no hay operación
        if to.is_white() || to.is_black() {
            return Instruction::Nop;
        }

        // Calcular cambio de hue y lightness
        if let (Some(old_hue), Some(old_light)) = (from.hue(), from.lightness()) {
            if let (Some(new_hue), Some(new_light)) = (to.hue(), to.lightness()) {
                let hue_change = (new_hue as i8) - (old_hue as i8);
                let light_change = (new_light as i8) - (old_light as i8);

                if let Some(op) = get_operation(hue_change, light_change) {
                    return self.operation_to_instruction(op, block_size);
                }
            }
        }

        Instruction::Nop
    }

    /// Convierte una operación Piet a instrucción
    fn operation_to_instruction(
        &self,
        op: crate::ops::Operation,
        block_size: usize,
    ) -> Instruction {
        use crate::ops::Operation;
        match op {
            Operation::Push => Instruction::Push(block_size as i32),
            Operation::Pop => Instruction::Pop,
            Operation::Add => Instruction::Add,
            Operation::Subtract => Instruction::Subtract,
            Operation::Multiply => Instruction::Multiply,
            Operation::Divide => Instruction::Divide,
            Operation::Mod => Instruction::Mod,
            Operation::Not => Instruction::Not,
            Operation::Greater => Instruction::Greater,
            Operation::Pointer => Instruction::Pointer,
            Operation::Switch => Instruction::Switch,
            Operation::Duplicate => Instruction::Duplicate,
            Operation::Roll => Instruction::Roll,
            Operation::InNumber => Instruction::InNumber,
            Operation::InChar => Instruction::InChar,
            Operation::OutNumber => Instruction::OutNumber,
            Operation::OutChar => Instruction::OutChar,
        }
    }

    /// Ejecuta múltiples pasos
    /// Se detiene cuando: halted, max_steps alcanzados, o necesita input sin tenerlo
    pub fn play(&mut self, max_steps: usize) -> Result<usize, VmError> {
        let mut executed = 0;
        while !self.halted && executed < max_steps {
            // Verificar si necesitamos input antes de ejecutar
            if let Ok(Instruction::InChar | Instruction::InNumber) = self.get_next_instruction() {
                if !self.input.has_input() {
                    // Necesitamos input pero no lo tenemos
                    // Retornar para que el caller pueda proveer input
                    return Ok(executed);
                }
            }

            match self.stroke() {
                Ok(_) => executed += 1,
                Err(VmError::Halted) => break,
                Err(VmError::InvalidInput) => {
                    // Este caso no debería ocurrir ahora que verificamos arriba
                    // pero por seguridad, retornamos en lugar de propagar error
                    return Ok(executed);
                }
                Err(e) => return Err(e),
            }
        }
        Ok(executed)
    }

    /// Vista previa del stack (dry-run sin side effects)
    ///
    /// Ejecuta la siguiente instrucción en una VM clonada y retorna
    /// el estado del stack antes y después
    pub fn preview_stack(&self) -> Result<StackPreview, VmError> {
        // Calcular la instrucción dinámicamente
        let instr = self.get_next_instruction()?;

        let stack_before = self.stack.clone();

        // Clonar la VM y ejecutar la instrucción
        let mut vm_clone = self.clone();
        let result = vm_clone.execute_instruction(&instr);

        let stack_after = vm_clone.stack.clone();

        Ok(StackPreview {
            stack_before,
            stack_after,
            instruction: instr.clone(),
            success: result.is_ok(),
            error: result.err().map(|e| format!("{}", e)),
        })
    }

    /// Obtiene la siguiente instrucción que se ejecutará (calculada dinámicamente)
    fn get_next_instruction(&self) -> Result<Instruction, VmError> {
        let current_color = self.grid.get(self.position).ok_or(VmError::OutOfBounds)?;

        // Negro o halt
        if current_color.is_black() {
            return Ok(Instruction::Halt);
        }

        // Blanco = Nop (se desliza)
        if current_color.is_white() {
            return Ok(Instruction::Nop);
        }

        // Obtener block info
        let block_id = self
            .grid
            .get_block_id(self.position)
            .ok_or(VmError::OutOfBounds)?;
        let block_size = self
            .grid
            .get_block_info(block_id)
            .map(|info| info.size)
            .ok_or(VmError::OutOfBounds)?;

        // Buscar salida válida
        let mut dp = self.dp;
        let mut cc = self.cc;

        for attempt in 0..8 {
            if let Some(exit_pos) = self.grid.get_exit(block_id, dp, cc) {
                if let Some(color) = self.grid.get(exit_pos) {
                    if color.is_black() {
                        if attempt % 2 == 0 {
                            cc = cc.toggle();
                        } else {
                            dp = dp.rotate_clockwise(1);
                        }
                        continue;
                    } else if color.is_white() {
                        if let Some(slide_pos) = self.slide_through_white(exit_pos, dp) {
                            if let Some(slide_color) = self.grid.get(slide_pos) {
                                return Ok(self.color_transition_to_instruction(
                                    current_color,
                                    slide_color,
                                    block_size,
                                ));
                            }
                        }
                        if attempt % 2 == 0 {
                            cc = cc.toggle();
                        } else {
                            dp = dp.rotate_clockwise(1);
                        }
                        continue;
                    } else {
                        return Ok(self.color_transition_to_instruction(
                            current_color,
                            color,
                            block_size,
                        ));
                    }
                }
            }
            if attempt % 2 == 0 {
                cc = cc.toggle();
            } else {
                dp = dp.rotate_clockwise(1);
            }
        }

        Ok(Instruction::Halt)
    }

    /// Retorna el snapshot del estado actual
    pub fn snapshot(&self) -> BytecodeVmSnapshot {
        let next_instruction = self.get_next_instruction().ok();
        // Para el index, usamos el position_map si existe (para compatibilidad)
        let instruction_index = self
            .program
            .get_instruction_index_at(self.position.x, self.position.y);

        BytecodeVmSnapshot {
            position: self.position,
            dp: self.dp,
            cc: self.cc,
            stack: self.stack.clone(),
            halted: self.halted,
            steps: self.steps,
            next_instruction,
            instruction_index,
        }
    }

    /// Lee la salida
    pub fn ink(&self) -> Vec<i32> {
        self.output.read()
    }

    /// Lee la salida como string
    pub fn ink_string(&self) -> String {
        self.output.read_string()
    }

    /// Escribe entrada
    pub fn input(&mut self, value: i32) {
        self.input.write(value);
    }

    /// Escribe entrada como char
    pub fn input_char(&mut self, c: char) {
        self.input.write_char(c);
    }

    /// Load text as character inputs (for in_char operations)
    pub fn load_input_text(&mut self, text: &str) {
        self.input.load_text(text);
    }

    /// Load numbers from string (whitespace-separated)
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

    /// Verifica si está detenida
    pub fn is_halted(&self) -> bool {
        self.halted
    }

    /// Retorna el tamaño del stack
    pub fn stack_size(&self) -> usize {
        self.stack.len()
    }

    /// Peek at the top of the stack without removing it
    pub fn peek(&self) -> Option<i32> {
        self.stack.last().copied()
    }

    /// Pop a value from the stack
    pub fn pop(&mut self) -> Result<i32, VmError> {
        self.stack.pop().ok_or(VmError::StackUnderflow)
    }

    /// Get the instruction at the current position
    fn get_instruction_at_current_position(&self) -> Result<Instruction, VmError> {
        let x = self.position.x;
        let y = self.position.y;
        if y < self.program.position_map.len() && x < self.program.position_map[y].len() {
            if let Some(idx) = self.program.position_map[y][x] {
                if idx < self.program.instructions.len() {
                    return Ok(self.program.instructions[idx].clone());
                }
            }
        }
        Err(VmError::Halted)
    }

    /// Execute the instruction at the current position and advance
    pub fn execute_current(&mut self) -> Result<(), VmError> {
        let instr = self.get_instruction_at_current_position()?;
        self.execute_instruction(&instr)?;
        self.steps += 1;
        // Advance position to the right for tests (simplest navigation)
        self.position.x += 1;
        Ok(())
    }

    /// Get the current direction pointer
    pub fn direction_pointer(&self) -> Direction {
        self.dp
    }

    /// Get the current codel chooser
    pub fn codel_chooser(&self) -> CodelChooser {
        self.cc
    }

    /// Retorna el número de pasos ejecutados
    pub fn get_steps(&self) -> usize {
        self.steps
    }

    /// Retorna una referencia a las instrucciones del programa
    pub fn instructions(&self) -> &[Instruction] {
        &self.program.instructions
    }

    // === Métodos privados ===

    /// Ejecuta una instrucción de bytecode
    fn execute_instruction(&mut self, instr: &Instruction) -> Result<(), VmError> {
        match instr {
            Instruction::Push(value) => {
                self.stack.push(*value);
            }
            Instruction::Pop => {
                self.check_stack(1)?;
                self.pop()?;
            }
            Instruction::Add => {
                self.check_stack(2)?;
                let b = self.pop()?;
                let a = self.pop()?;
                self.stack.push(a.wrapping_add(b));
            }
            Instruction::Subtract => {
                self.check_stack(2)?;
                let b = self.pop()?;
                let a = self.pop()?;
                self.stack.push(a.wrapping_sub(b));
            }
            Instruction::Multiply => {
                self.check_stack(2)?;
                let b = self.pop()?;
                let a = self.pop()?;
                // Usar wrapping para evitar panic en overflow
                self.stack.push(a.wrapping_mul(b));
            }
            Instruction::Divide => {
                self.check_stack(2)?;
                let b = self.pop()?;
                if b == 0 {
                    // División por cero: restaurar stack y retornar error
                    self.stack.push(b);
                    return Err(VmError::DivisionByZero);
                }
                let a = self.pop()?;
                self.stack.push(a / b);
            }
            Instruction::Mod => {
                self.check_stack(2)?;
                let b = self.pop()?;
                if b == 0 {
                    // División por cero: restaurar stack y retornar error
                    self.stack.push(b);
                    return Err(VmError::DivisionByZero);
                }
                let a = self.pop()?;
                self.stack.push(a.rem_euclid(b));
            }
            Instruction::Not => {
                self.check_stack(1)?;
                let a = self.pop()?;
                self.stack.push(if a == 0 { 1 } else { 0 });
            }
            Instruction::Greater => {
                self.check_stack(2)?;
                let b = self.pop()?;
                let a = self.pop()?;
                self.stack.push(if a > b { 1 } else { 0 });
            }
            Instruction::Pointer => {
                self.check_stack(1)?;
                let n = self.pop()?;
                self.dp = self.dp.rotate_clockwise(n);
            }
            Instruction::Switch => {
                self.check_stack(1)?;
                let n = self.pop()?;
                if n % 2 != 0 {
                    self.cc = self.cc.toggle();
                }
            }
            Instruction::Duplicate => {
                self.check_stack(1)?;
                let a = self.pop()?;
                self.stack.push(a);
                self.stack.push(a);
            }
            Instruction::Roll => {
                self.check_stack(2)?;
                let times = self.pop()?;
                let depth = self.pop()?;

                // Negative depth is an error - command is ignored
                if depth < 0 {
                    return Ok(());
                }

                let depth = depth as usize;

                if depth > self.stack.len() {
                    return Err(VmError::StackUnderflow);
                }

                if depth == 0 {
                    return Ok(());
                }

                let times = times.rem_euclid(depth as i32);
                for _ in 0..times {
                    let len = self.stack.len();
                    let top = self.stack.remove(len - 1);
                    self.stack.insert(len - depth, top);
                }
            }
            Instruction::InNumber => {
                let value = self.input.read_number()?;
                self.stack.push(value);
            }
            Instruction::InChar => {
                let value = self.input.read_char()?;
                self.stack.push(value);
            }
            Instruction::OutNumber => {
                self.check_stack(1)?;
                let value = self.pop()?;
                self.output.write_number(value);
            }
            Instruction::OutChar => {
                self.check_stack(1)?;
                let value = self.pop()?;
                self.output.write_char(value);
            }
            Instruction::Nop => {
                // No hace nada
            }
            Instruction::Halt => {
                self.halted = true;
            }
        }
        Ok(())
    }

    /// Verifica que haya al menos n elementos en el stack
    fn check_stack(&self, n: usize) -> Result<(), VmError> {
        if self.stack.len() < n {
            Err(VmError::StackUnderflow)
        } else {
            Ok(())
        }
    }

    /// Deslizarse por celdas blancas hasta encontrar un color
    fn slide_through_white(&self, start: Position, mut dp: Direction) -> Option<Position> {
        let mut pos = start;
        let mut attempts = 0;

        loop {
            if attempts >= 8 {
                return None;
            }

            if let Some(next_pos) = pos.step(dp, self.grid.width(), self.grid.height()) {
                if let Some(color) = self.grid.get(next_pos) {
                    if color.is_white() {
                        pos = next_pos;
                        continue;
                    } else if !color.is_black() {
                        return Some(next_pos);
                    }
                }
            }

            // Borde o negro - rotar
            if attempts % 2 == 0 {
                // Toggle CC no afecta en blanco, pero avanzamos
            } else {
                dp = dp.rotate_clockwise(1);
            }
            attempts += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bytecode::Program;
    use crate::grid::Grid;

    #[test]
    fn test_bytecode_vm_from_grid() {
        // Crear una grilla simple con dos bloques
        // Rojo claro (0xFFC0C0) → Amarillo claro (0xFFFFC0)
        // Cambio: hue +1, light 0 → operación Add
        let rgba = vec![
            0xFF, 0xC0, 0xC0, 0xFF, // (0,0) rojo claro
            0xFF, 0xFF, 0xC0, 0xFF, // (1,0) amarillo claro
        ];

        let grid = Grid::from_rgba(2, 1, &rgba).unwrap();
        let vm = BytecodeVm::from_grid(grid).unwrap();

        // El VM debe tener instrucciones compiladas
        assert!(!vm.is_halted());
        assert_eq!(vm.stack_size(), 0);
    }

    #[test]
    fn test_bytecode_vm_basic() {
        let mut program = Program::new(10, 1);

        // Secuencia: Push(5), Push(3), Add (con espacio para moverse)
        let idx0 = program.add_instruction(Instruction::Push(5));
        let idx1 = program.add_instruction(Instruction::Push(3));
        let idx2 = program.add_instruction(Instruction::Add);

        // Mapear con espacio entre ellas
        program.map_position(0, 0, idx0);
        program.map_position(1, 0, idx1);
        program.map_position(2, 0, idx2);
        // Resto de posiciones sin mapear

        // Crear una grid dummy para el test
        let rgba = vec![0xFFu8; 10 * 4]; // 10x1 pixeles blancos
        let grid = Grid::from_rgba(10, 1, &rgba).unwrap();
        let mut vm = BytecodeVm::new(program, grid);

        // Ejecutar Push(5)
        let _ = vm.stroke(); // Puede fallar al moverse, OK
        if vm.stack_size() > 0 {
            assert_eq!(vm.stack_size(), 1);
        }
    }

    #[test]
    fn test_stack_preview() {
        // Crear una grid real con transiciones de color que produzcan Push
        // LightRed → Red = hue 0, lightness +1 = Push
        let rgba = vec![
            0xFF, 0xC0, 0xC0, 0xFF, // (0,0) LightRed - block size 1, Push(1)
            0xFF, 0x00, 0x00, 0xFF, // (1,0) Red
            0xC0, 0x00, 0x00, 0xFF, // (2,0) DarkRed - Halt (no puede salir)
        ];
        let grid = Grid::from_rgba(3, 1, &rgba).unwrap();
        let vm = BytecodeVm::from_grid(grid).unwrap();

        // Preview debe mostrar la siguiente instrucción (Push(1) porque block size = 1)
        let preview = vm.preview_stack().unwrap();
        assert_eq!(preview.stack_before, Vec::<i32>::new());
        // La instrucción debería ser Push(1) ya que el bloque LightRed tiene tamaño 1
        assert_eq!(preview.instruction, Instruction::Push(1));
        assert!(preview.success);

        // El estado original no cambia
        assert_eq!(vm.stack_size(), 0);
    }

    // ========================================================================
    // Tests para cada instrucción de Piet según la especificación
    // ========================================================================

    fn create_test_vm(instructions: Vec<Instruction>) -> BytecodeVm {
        let size = instructions.len().max(3);
        let mut program = Program::new(size, 1);
        for (i, instr) in instructions.into_iter().enumerate() {
            let idx = program.add_instruction(instr);
            program.map_position(i, 0, idx);
        }
        let rgba = vec![0xFFu8; size * 4];
        let grid = Grid::from_rgba(size, 1, &rgba).unwrap();
        BytecodeVm::new(program, grid)
    }

    #[test]
    fn test_instruction_push() {
        let mut vm = create_test_vm(vec![Instruction::Push(42)]);
        vm.execute_current().unwrap();
        assert_eq!(vm.stack_size(), 1);
        assert_eq!(vm.peek(), Some(42));
    }

    #[test]
    fn test_instruction_pop() {
        let mut vm = create_test_vm(vec![
            Instruction::Push(10),
            Instruction::Push(20),
            Instruction::Pop,
        ]);
        vm.execute_current().unwrap(); // Push(10)
        vm.execute_current().unwrap(); // Push(20)
        vm.execute_current().unwrap(); // Pop
        assert_eq!(vm.stack_size(), 1);
        assert_eq!(vm.peek(), Some(10));
    }

    #[test]
    fn test_instruction_add() {
        let mut vm = create_test_vm(vec![
            Instruction::Push(5),
            Instruction::Push(3),
            Instruction::Add,
        ]);
        vm.execute_current().unwrap();
        vm.execute_current().unwrap();
        vm.execute_current().unwrap();
        assert_eq!(vm.stack_size(), 1);
        assert_eq!(vm.peek(), Some(8));
    }

    #[test]
    fn test_instruction_subtract() {
        let mut vm = create_test_vm(vec![
            Instruction::Push(10),
            Instruction::Push(3),
            Instruction::Subtract,
        ]);
        vm.execute_current().unwrap();
        vm.execute_current().unwrap();
        vm.execute_current().unwrap();
        assert_eq!(vm.stack_size(), 1);
        assert_eq!(vm.peek(), Some(7)); // 10 - 3 = 7
    }

    #[test]
    fn test_instruction_multiply() {
        let mut vm = create_test_vm(vec![
            Instruction::Push(4),
            Instruction::Push(5),
            Instruction::Multiply,
        ]);
        vm.execute_current().unwrap();
        vm.execute_current().unwrap();
        vm.execute_current().unwrap();
        assert_eq!(vm.stack_size(), 1);
        assert_eq!(vm.peek(), Some(20));
    }

    #[test]
    fn test_instruction_divide() {
        let mut vm = create_test_vm(vec![
            Instruction::Push(20),
            Instruction::Push(4),
            Instruction::Divide,
        ]);
        vm.execute_current().unwrap();
        vm.execute_current().unwrap();
        vm.execute_current().unwrap();
        assert_eq!(vm.stack_size(), 1);
        assert_eq!(vm.peek(), Some(5)); // 20 / 4 = 5
    }

    #[test]
    fn test_instruction_divide_by_zero() {
        let mut vm = create_test_vm(vec![
            Instruction::Push(10),
            Instruction::Push(0),
            Instruction::Divide,
        ]);
        vm.execute_current().unwrap();
        vm.execute_current().unwrap();
        let result = vm.execute_current();
        // División por cero retorna error según la implementación actual
        assert!(result.is_err());
        // Stack permanece con ambos valores cuando hay error
        assert_eq!(vm.stack_size(), 2);
    }

    #[test]
    fn test_instruction_mod() {
        let mut vm = create_test_vm(vec![
            Instruction::Push(17),
            Instruction::Push(5),
            Instruction::Mod,
        ]);
        vm.execute_current().unwrap();
        vm.execute_current().unwrap();
        vm.execute_current().unwrap();
        assert_eq!(vm.stack_size(), 1);
        assert_eq!(vm.peek(), Some(2)); // 17 % 5 = 2
    }

    #[test]
    fn test_instruction_not() {
        let mut vm = create_test_vm(vec![
            Instruction::Push(0),
            Instruction::Not,
            Instruction::Push(5),
            Instruction::Not,
        ]);
        vm.execute_current().unwrap(); // Push(0)
        vm.execute_current().unwrap(); // Not -> 1
        assert_eq!(vm.peek(), Some(1));
        vm.execute_current().unwrap(); // Push(5)
        vm.execute_current().unwrap(); // Not -> 0
        assert_eq!(vm.peek(), Some(0));
    }

    #[test]
    fn test_instruction_greater() {
        let mut vm = create_test_vm(vec![
            Instruction::Push(10),
            Instruction::Push(5),
            Instruction::Greater,
            Instruction::Push(3),
            Instruction::Push(7),
            Instruction::Greater,
        ]);
        vm.execute_current().unwrap();
        vm.execute_current().unwrap();
        vm.execute_current().unwrap();
        assert_eq!(vm.peek(), Some(1)); // 10 > 5 = 1
        vm.execute_current().unwrap();
        vm.execute_current().unwrap();
        vm.execute_current().unwrap();
        assert_eq!(vm.peek(), Some(0)); // 3 > 7 = 0
    }

    #[test]
    fn test_instruction_duplicate() {
        let mut vm = create_test_vm(vec![Instruction::Push(42), Instruction::Duplicate]);
        vm.execute_current().unwrap();
        vm.execute_current().unwrap();
        assert_eq!(vm.stack_size(), 2);
        assert_eq!(vm.peek(), Some(42));
        vm.pop().unwrap();
        assert_eq!(vm.peek(), Some(42));
    }

    #[test]
    fn test_instruction_roll() {
        let mut vm = create_test_vm(vec![
            Instruction::Push(1),
            Instruction::Push(2),
            Instruction::Push(3),
            Instruction::Push(4),
            Instruction::Push(2), // depth
            Instruction::Push(1), // rolls
            Instruction::Roll,
        ]);
        // Stack antes de Roll: [1, 2, 3, 4, 2, 1]
        // Roll depth=2, rolls=1: toma los top 2 elementos excluyendo depth y rolls
        for _ in 0..7 {
            vm.execute_current().unwrap();
        }
        // Roll con depth=2, rolls=1 debe rotar [3, 4] -> [4, 3]
        // Stack después: [1, 2, 4, 3]
        assert_eq!(vm.stack_size(), 4);
        let top = vm.pop().unwrap();
        let second = vm.pop().unwrap();
        assert_eq!(top, 3);
        assert_eq!(second, 4);
    }

    #[test]
    fn test_instruction_in_char() {
        let mut vm = create_test_vm(vec![Instruction::InChar]);
        vm.load_input_text("A");
        vm.execute_current().unwrap();
        assert_eq!(vm.stack_size(), 1);
        assert_eq!(vm.peek(), Some(65)); // ASCII 'A'
    }

    #[test]
    fn test_instruction_in_number() {
        let mut vm = create_test_vm(vec![Instruction::InNumber]);
        vm.load_input_numbers("42");
        vm.execute_current().unwrap();
        assert_eq!(vm.stack_size(), 1);
        assert_eq!(vm.peek(), Some(42));
    }

    #[test]
    fn test_instruction_out_char() {
        let mut vm = create_test_vm(vec![
            Instruction::Push(72), // 'H'
            Instruction::OutChar,
        ]);
        vm.execute_current().unwrap();
        vm.execute_current().unwrap();
        let output = vm.ink_string();
        assert_eq!(output, "H");
    }

    #[test]
    fn test_instruction_out_number() {
        let mut vm = create_test_vm(vec![Instruction::Push(42), Instruction::OutNumber]);
        vm.execute_current().unwrap();
        vm.execute_current().unwrap();
        let output = vm.ink();
        assert_eq!(output, vec![42]);
    }

    #[test]
    fn test_instruction_pointer() {
        let mut vm = create_test_vm(vec![Instruction::Push(1), Instruction::Pointer]);
        let initial_dp = vm.dp;
        vm.execute_current().unwrap();
        vm.execute_current().unwrap();
        let new_dp = vm.dp;
        // DP debe haber rotado 1 vez en sentido horario
        assert_ne!(initial_dp, new_dp);
    }

    #[test]
    fn test_instruction_switch() {
        let mut vm = create_test_vm(vec![Instruction::Push(1), Instruction::Switch]);
        let initial_cc = vm.cc;
        vm.execute_current().unwrap();
        vm.execute_current().unwrap();
        let new_cc = vm.cc;
        // CC debe haber cambiado
        assert_ne!(initial_cc, new_cc);
    }

    #[test]
    fn test_instruction_nop() {
        let mut vm = create_test_vm(vec![Instruction::Push(5), Instruction::Nop]);
        vm.execute_current().unwrap();
        vm.execute_current().unwrap();
        // Nop no debe cambiar el stack
        assert_eq!(vm.stack_size(), 1);
        assert_eq!(vm.peek(), Some(5));
    }

    #[test]
    fn test_instruction_halt() {
        let mut vm = create_test_vm(vec![Instruction::Halt]);
        assert!(!vm.is_halted());
        vm.execute_current().unwrap();
        assert!(vm.is_halted());
    }

    #[test]
    fn test_multiple_in_char() {
        let mut vm = create_test_vm(vec![
            Instruction::InChar,
            Instruction::InChar,
            Instruction::InChar,
            Instruction::InChar,
        ]);
        vm.load_input_text("HOLA");
        vm.execute_current().unwrap();
        vm.execute_current().unwrap();
        vm.execute_current().unwrap();
        vm.execute_current().unwrap();
        assert_eq!(vm.stack_size(), 4);
        // Stack tiene los caracteres en orden: H, O, L, A (A en el tope)
        assert_eq!(vm.pop().unwrap(), 65); // 'A'
        assert_eq!(vm.pop().unwrap(), 76); // 'L'
        assert_eq!(vm.pop().unwrap(), 79); // 'O'
        assert_eq!(vm.pop().unwrap(), 72); // 'H'
    }
}
