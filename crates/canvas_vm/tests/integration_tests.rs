/// Integration tests usando ejemplos PNG de Piet
use canvas_vm::{BytecodeVm, Grid};
use image::ImageReader;
use std::path::PathBuf;

/// Helper para obtener la ruta al workspace root
fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

/// Helper para cargar una imagen PNG y crear una VM
fn load_piet_image(relative_path: &str) -> BytecodeVm {
    let path = workspace_root().join(relative_path);
    
    let img = ImageReader::open(&path)
        .unwrap_or_else(|_| panic!("Failed to open image at {:?}", path))
        .decode()
        .expect("Failed to decode image")
        .to_rgba8();

    let (width, height) = img.dimensions();
    let rgba_data: Vec<u8> = img.into_raw();

    let grid = Grid::from_rgba(width as usize, height as usize, &rgba_data)
        .expect("Failed to create grid");

    BytecodeVm::from_grid(grid).expect("Failed to create VM")
}

#[test]
fn test_hello_world_compiles() {
    let vm = load_piet_image("tools/fixtures/samples/HelloWorld.png");
    
    // Verificar que el programa se compiló sin errores
    assert!(!vm.is_halted(), "VM should start in non-halted state");
    assert_eq!(vm.stack_size(), 0, "Stack should be empty initially");
}

#[test]
fn test_hello_world_execution() {
    let mut vm = load_piet_image("tools/fixtures/samples/HelloWorld.png");
    
    // Ejecutar hasta que se detenga o máximo 1000 pasos
    let max_steps = 1000;
    let mut steps = 0;
    
    while !vm.is_halted() && steps < max_steps {
        match vm.stroke() {
            Ok(_) => steps += 1,
            Err(_) => break,
        }
    }
    
    // Verificar que produjo alguna salida
    let output = vm.ink_string();
    println!("HelloWorld output: {:?}", output);
    
    // HelloWorld debería producir texto
    assert!(!output.is_empty(), "HelloWorld should produce output");
}

#[test]
fn test_hello_world2_execution() {
    let mut vm = load_piet_image("tools/fixtures/samples/HelloWorld2.png");
    
    let max_steps = 5000;
    let mut steps = 0;
    
    while !vm.is_halted() && steps < max_steps {
        match vm.stroke() {
            Ok(_) => steps += 1,
            Err(_) => break,
        }
    }
    
    let output = vm.ink_string();
    println!("HelloWorld2 output: {:?}", output);
    println!("HelloWorld2 executed {} steps", steps);
}

#[test]
fn test_hello_world3_execution() {
    let mut vm = load_piet_image("tools/fixtures/samples/HelloWorld3.png");
    
    let max_steps = 5000;
    let mut steps = 0;
    
    while !vm.is_halted() && steps < max_steps {
        match vm.stroke() {
            Ok(_) => steps += 1,
            Err(_) => break,
        }
    }
    
    let output = vm.ink_string();
    println!("HelloWorld3 output: {:?}", output);
    println!("HelloWorld3 executed {} steps", steps);
}

#[test]
fn test_pi_execution() {
    let mut vm = load_piet_image("tools/fixtures/samples/PI.png");
    
    let max_steps = 10000;
    let mut steps = 0;
    
    while !vm.is_halted() && steps < max_steps {
        match vm.stroke() {
            Ok(_) => steps += 1,
            Err(_) => break,
        }
    }
    
    let output = vm.ink_string();
    println!("PI output: {:?}", output);
    println!("PI executed {} steps", steps);
}

#[test]
fn test_piet_compiles() {
    let vm = load_piet_image("tools/fixtures/samples/Piet.png");
    
    assert!(!vm.is_halted(), "VM should start in non-halted state");
}

#[test]
fn test_piet_execution() {
    let mut vm = load_piet_image("tools/fixtures/samples/Piet.png");
    
    let max_steps = 5000;
    let mut steps = 0;
    
    while !vm.is_halted() && steps < max_steps {
        match vm.stroke() {
            Ok(_) => steps += 1,
            Err(_) => break,
        }
    }
    
    println!("Piet executed {} steps", steps);
    
    let output = vm.ink_string();
    println!("Piet output: {:?}", output);
    
    assert!(steps > 0, "Piet should execute some steps");
}

#[test]
fn test_prime_generator_compiles() {
    let vm = load_piet_image("tools/fixtures/samples/PrimeGenerator.png");
    
    assert!(!vm.is_halted(), "VM should start in non-halted state");
}

#[test]
fn test_prime_generator_execution() {
    let mut vm = load_piet_image("tools/fixtures/samples/PrimeGenerator.png");
    
    // Los generadores pueden correr indefinidamente, limitamos los pasos
    let max_steps = 10000;
    let mut steps = 0;
    
    while !vm.is_halted() && steps < max_steps {
        match vm.stroke() {
            Ok(_) => steps += 1,
            Err(_) => break,
        }
    }
    
    println!("PrimeGenerator executed {} steps", steps);
    
    let output = vm.ink();
    println!("PrimeGenerator output: {:?}", output);
    
    assert!(steps > 0, "PrimeGenerator should execute some steps");
}

#[test]
fn test_all_examples_snapshot() {
    // Test que todos los ejemplos pueden crear snapshots
    let examples = [
        "tools/fixtures/samples/HelloWorld.png",
        "tools/fixtures/samples/HelloWorld2.png",
        "tools/fixtures/samples/HelloWorld3.png",
        "tools/fixtures/samples/Piet.png",
        "tools/fixtures/samples/PrimeGenerator.png",
        "tools/fixtures/samples/PI.png",
    ];
    
    for example in &examples {
        let mut vm = load_piet_image(example);
        
        // Snapshot inicial
        let snapshot = vm.snapshot();
        assert_eq!(snapshot.steps, 0);
        assert_eq!(snapshot.stack.len(), 0);
        assert!(!snapshot.halted);
        
        // Ejecutar un paso y snapshot de nuevo
        if vm.stroke().is_ok() {
            let snapshot2 = vm.snapshot();
            assert_eq!(snapshot2.steps, 1);
        }
        
        println!("✓ {} creates valid snapshots", example);
    }
}

#[test]
fn test_pi_debug() {
    let mut vm = load_piet_image("tools/fixtures/samples/PI.png");
    
    println!("\n=== PI.png Debug Execution ===\n");
    
    let max_steps = 50;
    let mut steps = 0;
    
    while !vm.is_halted() && steps < max_steps {
        let snapshot = vm.snapshot();
        println!("Step {:2}: pos=({:3},{:3}), dp={:?}, cc={:?}, stack={:?}, next={:?}", 
            steps, 
            snapshot.position.x, 
            snapshot.position.y,
            snapshot.dp,
            snapshot.cc,
            snapshot.stack,
            snapshot.next_instruction);
        
        match vm.stroke() {
            Ok(_) => steps += 1,
            Err(e) => {
                println!("Error: {:?}", e);
                break;
            }
        }
    }
    
    let final_snapshot = vm.snapshot();
    println!("\nFinal state: pos=({},{}), halted={}", 
        final_snapshot.position.x, final_snapshot.position.y, vm.is_halted());
    println!("Output: {:?}", vm.ink_string());
    println!("Total steps: {}", steps);
}

#[test]
fn test_bytecode_compilation_all_examples() {
    // Verificar que todos los ejemplos compilan a bytecode
    let examples = [
        "tools/fixtures/samples/HelloWorld.png",
        "tools/fixtures/samples/HelloWorld2.png",
        "tools/fixtures/samples/HelloWorld3.png",
        "tools/fixtures/samples/Piet.png",
        "tools/fixtures/samples/PrimeGenerator.png",
        "tools/fixtures/samples/PI.png",
    ];
    
    for example in &examples {
        let path = workspace_root().join(example);
        let img = ImageReader::open(&path)
            .unwrap_or_else(|_| panic!("Failed to open image at {:?}", path))
            .decode()
            .expect("Failed to decode image")
            .to_rgba8();

        let (width, height) = img.dimensions();
        let rgba_data: Vec<u8> = img.into_raw();

        let grid = Grid::from_rgba(width as usize, height as usize, &rgba_data)
            .expect("Failed to create grid");

        // BytecodeVm::from_grid compila internamente
        let vm = BytecodeVm::from_grid(grid);
        
        assert!(vm.is_ok(), "{} should compile successfully", example);
        
        println!("✓ {} compiles to bytecode", example);
    }
}
