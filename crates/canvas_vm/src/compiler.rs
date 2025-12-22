/// Compilador de Piet: Imagen → Bytecode
use crate::bytecode::{Instruction, Program};
use crate::error::VmError;
use crate::exits::{CodelChooser, Direction, Position};
use crate::grid::Grid;
use crate::ops::{get_operation, Operation, PietColor};
use std::collections::{HashMap, HashSet, VecDeque};

/// Compilador que transforma una grilla de Piet en bytecode
pub struct Compiler {
    grid: Grid,
}

impl Compiler {
    /// Crea un nuevo compilador con una grilla
    pub fn new(grid: Grid) -> Self {
        Self { grid }
    }

    /// Compila la grilla a un programa de bytecode
    /// 
    /// Estrategia:
    /// 1. Recorre todos los bloques posibles desde (0,0)
    /// 2. Para cada transición de color, genera la instrucción correspondiente
    /// 3. Mapea cada posición a su instrucción
    pub fn compile(&self) -> Result<Program, VmError> {
        let width = self.grid.width();
        let height = self.grid.height();
        let mut program = Program::new(width, height);
        
        // Mapa de (block_id, dp, cc) → instruction_index para evitar duplicados
        let mut block_instr_map: HashMap<(usize, Direction, CodelChooser), usize> = HashMap::new();
        
        // BFS para explorar todos los estados alcanzables desde (0,0)
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        
        // Estado inicial: posición (0,0), DP derecha, CC izquierda
        let start_pos = Position::new(0, 0);
        let start_dp = Direction::Right;
        let start_cc = CodelChooser::Left;
        
        queue.push_back((start_pos, start_dp, start_cc));
        visited.insert((start_pos, start_dp, start_cc));
        
        while let Some((pos, dp, cc)) = queue.pop_front() {
            // eprintln!("DEBUG compile: processing pos=({},{}) dp={:?} cc={:?}", pos.x, pos.y, dp, cc);
            
            let current_color = match self.grid.get(pos) {
                Some(color) => color,
                None => {
                    // eprintln!("DEBUG compile: no color at pos, skipping");
                    continue;
                }
            };
            
            // eprintln!("DEBUG compile: color={:?}", current_color);
            
            // Negro = halt
            if current_color.is_black() {
                let idx = program.add_instruction(Instruction::Halt);
                program.map_position(pos.x, pos.y, idx);
                continue;
            }
            
            // Blanco = deslizamiento (slide)
            // En Piet, los bloques blancos se "atraviesan" hasta encontrar color
            if current_color.is_white() {
                // Buscar la siguiente posición no-blanca
                if let Some(next_pos) = self.slide_preview(pos, dp) {
                    // Crear un NOP que apunta directamente al destino
                    let idx = program.add_instruction(Instruction::Nop);
                    program.map_position(pos.x, pos.y, idx);
                    program.map_next_position(pos.x, pos.y, next_pos.x, next_pos.y);
                    
                    let next_state = (next_pos, dp, cc);
                    if !visited.contains(&next_state) {
                        visited.insert(next_state);
                        queue.push_back(next_state);
                    }
                }
                // Si no hay siguiente posición válida, simplemente no agregamos nada
                // (el programa terminará cuando llegue aquí)
                continue;
            }
            
            // Color cromático: obtener bloque y calcular transición
            let block_id = match self.grid.get_block_id(pos) {
                Some(id) => id,
                None => continue,
            };
            
            let block_info = match self.grid.get_block_info(block_id) {
                Some(info) => info,
                None => continue,
            };
            
            // eprintln!("DEBUG compile: block_id={} block_size={}", block_id, block_info.size);
            
            // Verificar si ya compilamos esta transición
            let key = (block_id, dp, cc);
            if let Some(&instr_idx) = block_instr_map.get(&key) {
                // eprintln!("DEBUG compile: reusing existing instruction at index {}", instr_idx);
                // Ya existe, solo mapear la posición
                program.map_position(pos.x, pos.y, instr_idx);
                // Intentar mapear la siguiente posición también
                if let Some(next_pos) = self.grid.get_exit(block_id, dp, cc) {
                    program.map_next_position(pos.x, pos.y, next_pos.x, next_pos.y);
                }
                continue;
            }
            
            // Intentar encontrar una salida válida (con reintentos como Piet real)
            let exit_result = self.find_valid_exit(block_id, dp, cc);
            // eprintln!("DEBUG compile: find_valid_exit result={:?}", exit_result);
            
            if let Some((next_pos, exit_dp, exit_cc)) = exit_result {
                if let Some(next_color) = self.grid.get(next_pos) {
                    // Si la salida lleva a blanco, deslizarse hasta el siguiente color
                    let (final_pos, final_color) = if next_color.is_white() {
                        if let Some(slide_pos) = self.slide_preview(next_pos, exit_dp) {
                            // Nota: volver al mismo bloque después de slide ES válido en loops de Piet
                            // Solo es halt si slide_preview no encuentra ningún destino
                            if let Some(c) = self.grid.get(slide_pos) {
                                (slide_pos, c)
                            } else {
                                // No hay destino válido después del slide = halt
                                // eprintln!("DEBUG compile: no valid destination after slide, adding Halt");
                                let idx = program.add_instruction(Instruction::Halt);
                                for &block_pos in &block_info.positions {
                                    program.map_position(block_pos.x, block_pos.y, idx);
                                }
                                continue;
                            }
                        } else {
                            // Slide no encontró destino = halt
                            // eprintln!("DEBUG compile: slide found no destination, adding Halt");
                            let idx = program.add_instruction(Instruction::Halt);
                            for &block_pos in &block_info.positions {
                                program.map_position(block_pos.x, block_pos.y, idx);
                            }
                            continue;
                        }
                    } else {
                        (next_pos, next_color)
                    };
                    
                    // Calcular la operación basada en el cambio de color
                    let instr = self.color_transition_to_instruction(
                        current_color,
                        final_color,
                        block_info.size,
                    );
                    
                    let idx = program.add_instruction(instr.clone());
                    
                    // Mapear TODAS las posiciones del bloque a esta instrucción
                    for &block_pos in &block_info.positions {
                        program.map_position(block_pos.x, block_pos.y, idx);
                        program.map_next_position(block_pos.x, block_pos.y, final_pos.x, final_pos.y);
                    }
                    block_instr_map.insert(key, idx);
                    
                    // Para Switch y Pointer, explorar AMBAS ramas posibles
                    if matches!(instr, Instruction::Switch) {
                        // Rama 1: CC no cambia (valor par en stack)
                        let next_state_unchanged = (final_pos, exit_dp, exit_cc);
                        if !visited.contains(&next_state_unchanged) {
                            visited.insert(next_state_unchanged);
                            queue.push_back(next_state_unchanged);
                        }
                        // Rama 2: CC cambia (valor impar en stack)
                        let toggled_cc = exit_cc.toggle();
                        let next_state_toggled = (final_pos, exit_dp, toggled_cc);
                        if !visited.contains(&next_state_toggled) {
                            visited.insert(next_state_toggled);
                            queue.push_back(next_state_toggled);
                        }
                    } else if matches!(instr, Instruction::Pointer) {
                        // Pointer puede rotar 0, 1, 2, o 3 veces
                        for rotation in 0..4 {
                            let rotated_dp = exit_dp.rotate_clockwise(rotation);
                            let next_state = (final_pos, rotated_dp, exit_cc);
                            if !visited.contains(&next_state) {
                                visited.insert(next_state);
                                queue.push_back(next_state);
                            }
                        }
                    } else {
                        // Otras instrucciones: un solo siguiente estado
                        let next_state = (final_pos, exit_dp, exit_cc);
                        if !visited.contains(&next_state) {
                            visited.insert(next_state);
                            queue.push_back(next_state);
                        }
                    }
                }
            } else {
                // No hay salida después de 8 intentos = halt
                // eprintln!("DEBUG compile: no exit after 8 attempts, adding Halt for all block positions");
                let idx = program.add_instruction(Instruction::Halt);
                for &block_pos in &block_info.positions {
                    program.map_position(block_pos.x, block_pos.y, idx);
                }
            }
        }
        
        Ok(program)
    }
    
    /// Convierte una transición de color en una instrucción
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
    
    /// Convierte una operación de Piet en una instrucción de bytecode
    fn operation_to_instruction(&self, op: Operation, block_size: usize) -> Instruction {
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
    
    /// Busca una salida válida desde un bloque, rotando CC y DP como Piet real
    /// Retorna (next_pos, final_dp, final_cc) o None si no hay salida válida
    fn find_valid_exit(
        &self, 
        block_id: usize, 
        mut dp: Direction, 
        mut cc: CodelChooser
    ) -> Option<(Position, Direction, CodelChooser)> {
        // Piet intenta 8 veces: alterna entre rotar CC y rotar DP
        for attempt in 0..8 {
            if let Some(next_pos) = self.grid.get_exit(block_id, dp, cc) {
                return Some((next_pos, dp, cc));
            }
            
            // No hay salida, intentar otra dirección
            if attempt % 2 == 0 {
                // Intentos pares: rotar CC
                cc = cc.toggle();
            } else {
                // Intentos impares: rotar DP en sentido horario
                dp = dp.rotate_clockwise(1);
            }
        }
        
        // Después de 8 intentos, no hay salida = programa termina
        None
    }
    
    /// Vista previa de deslizamiento por blancos (sin modificar estado)
    fn slide_preview(&self, start_pos: Position, start_dp: Direction) -> Option<Position> {
        let mut pos = start_pos;
        let mut dp = start_dp;
        let mut attempts = 0;
        
        // Según las reglas de Piet:
        // - Cuando entras al blanco, el DP mantiene su dirección
        // - Deslízate por el blanco hasta encontrar un borde o negro
        // - Si encuentras borde/negro, rota CC, si sigue bloqueado, rota DP
        // - Después de 8 intentos sin salida, halt
        
        loop {
            if attempts >= 8 {
                // eprintln!("DEBUG slide_preview: 8 attempts reached, no exit");
                return None;
            }
            
            if let Some(next_pos) = pos.step(dp, self.grid.width(), self.grid.height()) {
                if let Some(color) = self.grid.get(next_pos) {
                    if color.is_white() {
                        // Continuar deslizándose por el blanco
                        pos = next_pos;
                        continue;
                    } else if !color.is_black() {
                        // Encontramos un bloque de color, salir aquí
                        // eprintln!("DEBUG slide_preview: found color at {:?}", next_pos);
                        return Some(next_pos);
                    }
                }
            }
            
            // Borde del canvas o bloque negro - rotar y reintentar
            // Piet: primero toggle CC, luego rotar DP
            if attempts % 2 == 0 {
                // Toggle CC (no afecta la dirección en blanco realmente)
                // Solo avanzamos al siguiente intento
            } else {
                // Rotar DP en sentido horario
                dp = dp.rotate_clockwise(1);
            }
            attempts += 1;
            // eprintln!("DEBUG slide_preview: attempt {} at {:?} dp={:?}", attempts, pos, dp);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_compilation() {
        // Crear una grilla simple: rojo (0xFFC0C0) → amarillo (0xFFFFC0)
        // Cambio: hue +1, light 0 → operación Add
        let rgba = vec![
            0xFF, 0xC0, 0xC0, 0xFF, // (0,0) rojo claro
            0xFF, 0xFF, 0xC0, 0xFF, // (1,0) amarillo claro
        ];
        
        let grid = Grid::from_rgba(2, 1, &rgba).unwrap();
        let compiler = Compiler::new(grid);
        let program = compiler.compile().unwrap();
        
        // Debe tener al menos una instrucción Add
        assert!(program.instructions.iter().any(|i| matches!(i, Instruction::Add)));
    }
}
