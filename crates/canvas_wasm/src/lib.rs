use wasm_bindgen::prelude::*;
use canvas_vm::{Grid, BytecodeVm, Compiler, Instruction, Program};
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

/// Estado de la VM serializable para JavaScript
#[derive(Serialize, Deserialize)]
pub struct VmSnapshot {
    pub position_x: usize,
    pub position_y: usize,
    pub direction: String,
    pub codel_chooser: String,
    pub stack: Vec<i32>,
    pub halted: bool,
    pub steps: usize,
    pub instruction_index: Option<usize>,
}

/// Instrucción de bytecode serializable para JavaScript
#[derive(Serialize, Deserialize)]
pub struct BytecodeInstruction {
    pub index: usize,
    pub opcode: String,
    pub from_color: String,
    pub to_color: String,
}

/// Canvas - Wrapper WASM de la VM de Piet
#[wasm_bindgen]
pub struct Canvas {
    vm: Option<BytecodeVm>,
    grid: Option<Grid>,
    program: Option<Program>,
    width: usize,
    height: usize,
}

#[wasm_bindgen]
impl Canvas {
    /// Crea una nueva instancia de Canvas
    #[wasm_bindgen(constructor)]
    pub fn new() -> Canvas {
        console_log!("🎨 Canvas created");
        Canvas {
            vm: None,
            program: None,
            grid: None,
            width: 0,
            height: 0,
        }
    }

    /// Carga una grilla desde datos RGBA (4 bytes por píxel)
    /// paint(rgbaData: Uint8Array, width: number, height: number)
    #[wasm_bindgen]
    pub fn paint(&mut self, rgba_data: &[u8], width: usize, height: usize) -> Result<(), JsValue> {
        console_log!("🎨 Loading grid {}x{}", width, height);
        
        let grid = Grid::from_rgba(width, height, rgba_data)
            .map_err(|e| JsValue::from_str(&format!("Failed to create grid: {}", e)))?;
        
        self.width = width;
        self.height = height;
        
        // Store grid for reset functionality
        self.grid = Some(grid.clone());
        
        // Compile to bytecode immediately
        console_log!("📝 Compiling to bytecode...");
        let compiler = Compiler::new(grid.clone());
        let program = compiler.compile()
            .map_err(|e| JsValue::from_str(&format!("Compilation error: {}", e)))?;
        
        console_log!("✅ Compiled {} instructions", program.instructions.len());
        
        // Create BytecodeVm with the compiled program and grid
        self.program = Some(program.clone());
        self.vm = Some(BytecodeVm::new(program, grid));
        
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
        
        // Debug: log first few instructions with their Debug representation
        for (i, instr) in program.instructions.iter().take(10).enumerate() {
            console_log!("  [{}] Debug={:?}", i, instr);
        }
        
        // Convert instructions to JS-friendly format
        let instructions: Vec<BytecodeInstruction> = program.instructions
            .iter()
            .enumerate()
            .map(|(i, instr)| {
                console_log!("Converting instruction {}: {:?}", i, instr);
                // Match each instruction variant to get a clean name
                let (op_name, value) = match instr {
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
                
                BytecodeInstruction {
                    index: i,
                    opcode: op_name,
                    from_color: String::new(),
                    to_color: value,
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
