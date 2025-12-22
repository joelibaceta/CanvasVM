/// Debug test para PrimeGenerator
use canvas_vm::{BytecodeVm, Grid};
use image::ImageReader;
use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

fn load_piet_image(relative_path: &str) -> (BytecodeVm, Grid) {
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

    let vm = BytecodeVm::from_grid(grid.clone()).expect("Failed to create VM");
    (vm, grid)
}

#[test]
fn debug_prime_generator() {
    let (mut vm, grid) = load_piet_image("tools/fixtures/samples/PrimeGenerator.png");
    
    println!("\n=== PrimeGenerator Debug ===");
    println!("Grid dimensions: {}x{}", grid.width(), grid.height());
    println!();
    
    // Ejecutar con log detallado
    let max_steps = 500000;
    let mut steps = 0;
    
    let mut instruction_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    let mut last_output_len = 0;
    
    while !vm.is_halted() && steps < max_steps {
        let snapshot = vm.snapshot();
        
        // Log la instrucción antes de ejecutar
        let instr_name = format!("{:?}", snapshot.next_instruction);
        *instruction_counts.entry(instr_name.clone()).or_insert(0) += 1;
        
        // Solo imprimir los primeros 100 pasos en detalle
        if steps < 100 {
            println!("Step {}: pos=({},{}) dp={:?} cc={:?} stack={:?} instr={:?}",
                steps,
                snapshot.position.x, snapshot.position.y,
                snapshot.dp, snapshot.cc,
                snapshot.stack,
                snapshot.next_instruction
            );
        }
        
        match vm.stroke() {
            Ok(_) => steps += 1,
            Err(e) => {
                println!("Error at step {}: {:?}", steps, e);
                break;
            }
        }
        
        // Verificar si hubo output
        let output = vm.ink();
        if output.len() > last_output_len {
            println!(">>> OUTPUT at step {}: {:?}", steps, &output[last_output_len..]);
            last_output_len = output.len();
        }
    }
    
    println!("\n=== Summary ===");
    println!("Total steps: {}", steps);
    println!("Halted: {}", vm.is_halted());
    println!("Stack size: {}", vm.stack_size());
    println!("Final stack (up to 50 elements): {:?}", &vm.snapshot().stack[..std::cmp::min(50, vm.stack_size())]);
    println!("Final output raw: {:?}", vm.ink());
    println!("Output as string: {:?}", vm.ink_string());
    
    println!("\n=== Instruction counts ===");
    let mut counts: Vec<_> = instruction_counts.iter().collect();
    counts.sort_by(|a, b| b.1.cmp(a.1));
    for (instr, count) in counts {
        println!("{}: {}", instr, count);
    }
}

#[test]
fn debug_erat2_official() {
    let (mut vm, grid) = load_piet_image("tools/fixtures/samples/erat2_official.png");
    
    println!("\n=== erat2 Official Debug ===");
    println!("Grid dimensions: {}x{}", grid.width(), grid.height());
    
    let max_steps = 100000;
    let mut steps = 0;
    let mut last_output_len = 0;
    
    while !vm.is_halted() && steps < max_steps {
        match vm.stroke() {
            Ok(_) => steps += 1,
            Err(_) => break,
        }
        
        let output = vm.ink();
        if output.len() > last_output_len {
            println!("OUTPUT at step {}: {:?} (string: {:?})", 
                steps, &output[last_output_len..],
                output[last_output_len..].iter()
                    .filter_map(|&v| char::from_u32(v as u32))
                    .collect::<String>());
            last_output_len = output.len();
        }
    }
    
    println!("\n=== Summary ===");
    println!("Total steps: {}", steps);
    println!("Halted: {}", vm.is_halted());
    println!("Stack size: {}", vm.stack_size());
    println!("Final output raw: {:?}", vm.ink());
    println!("Output as string: {:?}", vm.ink_string());
}
