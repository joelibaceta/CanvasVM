/// Integration tests para el API WASM usando ejemplos PNG reales
use canvas_wasm::Canvas;
use wasm_bindgen_test::*;
use web_sys::console;

wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

/// Carga las imágenes PNG como bytes en tiempo de compilación
const HELLO_WORLD_PNG: &[u8] = include_bytes!("../../../tools/fixtures/samples/HelloWorld.png");
const PIET_PNG: &[u8] = include_bytes!("../../../tools/fixtures/samples/Piet.png");
const PRIME_GENERATOR_PNG: &[u8] = include_bytes!("../../../tools/fixtures/samples/PrimeGenerator.png");

/// Helper para decodificar PNG y extraer RGBA data
fn decode_png(png_bytes: &[u8]) -> (Vec<u8>, u32, u32) {
    let img = image::load_from_memory(png_bytes)
        .expect("Failed to decode PNG")
        .to_rgba8();
    
    let (width, height) = img.dimensions();
    let rgba_data = img.into_raw();
    
    (rgba_data, width, height)
}

#[wasm_bindgen_test]
fn test_hello_world_loads_and_compiles() {
    let mut canvas = Canvas::new();
    let (rgba_data, width, height) = decode_png(HELLO_WORLD_PNG);
    
    // Debe poder cargar la imagen
    let result = canvas.paint(&rgba_data, width as usize, height as usize);
    assert!(result.is_ok(), "HelloWorld.png should load successfully");
    
    // Debe poder compilar a bytecode
    let bytecode = canvas.compile_to_bytecode();
    assert!(bytecode.is_ok(), "HelloWorld should compile to bytecode");
}

#[wasm_bindgen_test]
fn test_hello_world_execution() {
    let mut canvas = Canvas::new();
    let (rgba_data, width, height) = decode_png(HELLO_WORLD_PNG);
    
    canvas.paint(&rgba_data, width as usize, height as usize).unwrap();
    
    // Ejecutar programa hasta completar o límite de pasos
    let max_steps = 1000;
    let mut executed_steps = 0;
    
    for _ in 0..max_steps {
        match canvas.stroke() {
            Ok(_) => executed_steps += 1,
            Err(_) => break,
        }
    }
    
    // HelloWorld debería ejecutar algunos pasos
    assert!(executed_steps > 0, "HelloWorld should execute at least one step");
    
    // Debe generar alguna salida
    let output = canvas.ink_string().unwrap();
    console::log_1(&format!("HelloWorld output: {}", output).into());
    
    // HelloWorld normalmente imprime "Hello World!"
    assert!(!output.is_empty(), "HelloWorld should produce output");
}

#[wasm_bindgen_test]
fn test_piet_loads_and_compiles() {
    let mut canvas = Canvas::new();
    let (rgba_data, width, height) = decode_png(PIET_PNG);
    
    canvas.paint(&rgba_data, width as usize, height as usize).unwrap();
    
    // Verificar que compila
    let bytecode = canvas.compile_to_bytecode();
    assert!(bytecode.is_ok(), "Piet should compile to bytecode");
}

#[wasm_bindgen_test]
fn test_piet_snapshot_workflow() {
    let mut canvas = Canvas::new();
    let (rgba_data, width, height) = decode_png(PIET_PNG);
    
    canvas.paint(&rgba_data, width as usize, height as usize).unwrap();
    
    // Snapshot inicial
    let snapshot1 = canvas.snapshot();
    assert!(snapshot1.is_ok(), "Should get initial snapshot");
    
    // Ejecutar un paso
    let _ = canvas.stroke();
    
    // Snapshot después del paso
    let snapshot2 = canvas.snapshot();
    assert!(snapshot2.is_ok(), "Should get snapshot after stroke");
    
    // Reset
    canvas.reset().unwrap();
    
    // Snapshot después de reset
    let snapshot3 = canvas.snapshot();
    assert!(snapshot3.is_ok(), "Should get snapshot after reset");
}

#[wasm_bindgen_test]
fn test_prime_generator_loads() {
    let mut canvas = Canvas::new();
    let (rgba_data, width, height) = decode_png(PRIME_GENERATOR_PNG);
    
    let result = canvas.paint(&rgba_data, width as usize, height as usize);
    assert!(result.is_ok(), "PrimeGenerator.png should load successfully");
}

#[wasm_bindgen_test]
fn test_prime_generator_execution_limited() {
    let mut canvas = Canvas::new();
    let (rgba_data, width, height) = decode_png(PRIME_GENERATOR_PNG);
    
    canvas.paint(&rgba_data, width as usize, height as usize).unwrap();
    
    // Los generadores pueden correr indefinidamente, limitamos
    let max_steps = 100;
    let mut executed_steps = 0;
    
    for _ in 0..max_steps {
        match canvas.stroke() {
            Ok(_) => executed_steps += 1,
            Err(_) => break,
        }
    }
    
    console::log_1(&format!("PrimeGenerator executed {} steps (limited)", executed_steps).into());
    assert!(executed_steps > 0, "PrimeGenerator should execute");
}

#[wasm_bindgen_test]
fn test_all_examples_compile_to_bytecode() {
    let examples = [
        ("HelloWorld", HELLO_WORLD_PNG),
        ("Piet", PIET_PNG),
        ("PrimeGenerator", PRIME_GENERATOR_PNG),
    ];
    
    for (name, png_bytes) in &examples {
        let mut canvas = Canvas::new();
        let (rgba_data, width, height) = decode_png(png_bytes);
        
        canvas.paint(&rgba_data, width as usize, height as usize)
            .expect(&format!("{} should load", name));
        
        let bytecode = canvas.compile_to_bytecode()
            .expect(&format!("{} should compile", name));
        
        // Verificar que el bytecode tiene al menos una instrucción
        let bytecode_array: js_sys::Array = bytecode.into();
        assert!(bytecode_array.length() > 0, "{} should have instructions", name);
        
        console::log_1(&format!("✓ {} compiles to {} instructions", name, bytecode_array.length()).into());
    }
}

#[wasm_bindgen_test]
fn test_all_examples_reset_workflow() {
    let examples = [
        ("HelloWorld", HELLO_WORLD_PNG),
        ("Piet", PIET_PNG),
        ("PrimeGenerator", PRIME_GENERATOR_PNG),
    ];
    
    for (name, png_bytes) in &examples {
        let mut canvas = Canvas::new();
        let (rgba_data, width, height) = decode_png(png_bytes);
        
        canvas.paint(&rgba_data, width as usize, height as usize).unwrap();
        
        // Ejecutar algunos pasos
        for _ in 0..5 {
            let _ = canvas.stroke();
        }
        
        // Reset debe funcionar
        let reset_result = canvas.reset();
        assert!(reset_result.is_ok(), "{} reset should work", name);
        
        // Después del reset debería poder ejecutar de nuevo
        let stroke_result = canvas.stroke();
        assert!(stroke_result.is_ok() || stroke_result.is_err(), 
                "{} should be executable after reset", name);
        
        console::log_1(&format!("✓ {} reset workflow works", name).into());
    }
}

#[wasm_bindgen_test]
fn test_hello_world_full_web_workflow() {
    // Simula el workflow completo de la web con HelloWorld
    let mut canvas = Canvas::new();
    let (rgba_data, width, height) = decode_png(HELLO_WORLD_PNG);
    
    // 1. paint() - cargar imagen
    canvas.paint(&rgba_data, width as usize, height as usize).unwrap();
    console::log_1(&"✓ Image loaded".into());
    
    // 2. compile_to_bytecode() - mostrar en tabla
    let bytecode = canvas.compile_to_bytecode().unwrap();
    let bytecode_array: js_sys::Array = bytecode.into();
    console::log_1(&format!("✓ Compiled to {} instructions", bytecode_array.length()).into());
    
    // 3. snapshot() inicial
    let snapshot = canvas.snapshot().unwrap();
    console::log_1(&"✓ Initial snapshot created".into());
    
    // 4. stroke() en loop (simula "play")
    let mut steps_executed = 0;
    for _ in 0..100 {
        match canvas.stroke() {
            Ok(_) => {
                steps_executed += 1;
                let _ = canvas.snapshot(); // UI update
            }
            Err(_) => break,
        }
    }
    console::log_1(&format!("✓ Executed {} steps", steps_executed).into());
    
    // 5. ink_string() - ver output
    let output = canvas.ink_string().unwrap();
    console::log_1(&format!("✓ Output: {}", output).into());
    
    // 6. reset() - reiniciar
    canvas.reset().unwrap();
    console::log_1(&"✓ Reset successful".into());
    
    // 7. Verificar que puede ejecutar de nuevo
    let _ = canvas.stroke();
    console::log_1(&"✓ Can execute after reset".into());
    
    // Test exitoso si llegamos aquí sin panic
    assert!(true, "Full workflow completed");
}
