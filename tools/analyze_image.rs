//! Analiza una imagen Piet y muestra el bytecode generado
//! Compilar: rustc --edition 2021 -L target/release/deps tools/analyze_image.rs -o analyze_image
//! O usar: cargo build --release && rustc ...

use std::fs::File;
use std::io::Read;

fn main() {
    // Leer el archivo BMP manualmente
    let mut file = File::open("tools/fixtures/samples/echo4.bmp").expect("No se pudo abrir el archivo");
    let mut data = Vec::new();
    file.read_to_end(&mut data).expect("No se pudo leer el archivo");
    
    // BMP header parsing
    if &data[0..2] != b"BM" {
        panic!("No es un archivo BMP válido");
    }
    
    let pixel_offset = u32::from_le_bytes([data[10], data[11], data[12], data[13]]) as usize;
    let width = i32::from_le_bytes([data[18], data[19], data[20], data[21]]) as usize;
    let height = i32::from_le_bytes([data[22], data[23], data[24], data[25]]).abs() as usize;
    let bits_per_pixel = u16::from_le_bytes([data[28], data[29]]) as usize;
    
    println!("BMP Info:");
    println!("  Tamaño: {}x{}", width, height);
    println!("  Bits por pixel: {}", bits_per_pixel);
    println!("  Offset de datos: {}", pixel_offset);
    
    // Leer píxeles (BMP es BGR, bottom-up)
    let row_size = ((width * 3 + 3) / 4) * 4;
    println!("\nColores (de izquierda a derecha):");
    
    // BMP es bottom-up, así que la fila 0 está al final
    let row_start = pixel_offset;
    for x in 0..width {
        let offset = row_start + x * 3;
        let b = data[offset];
        let g = data[offset + 1];
        let r = data[offset + 2];
        
        let color_name = match (r, g, b) {
            (0xFF, 0xC0, 0xC0) => "Light Red",
            (0xFF, 0xFF, 0xC0) => "Light Yellow",
            (0xC0, 0xFF, 0xC0) => "Light Green",
            (0xC0, 0xFF, 0xFF) => "Light Cyan",
            (0xC0, 0xC0, 0xFF) => "Light Blue",
            (0xFF, 0xC0, 0xFF) => "Light Magenta",
            (0xFF, 0x00, 0x00) => "Red",
            (0xFF, 0xFF, 0x00) => "Yellow",
            (0x00, 0xFF, 0x00) => "Green",
            (0x00, 0xFF, 0xFF) => "Cyan",
            (0x00, 0x00, 0xFF) => "Blue",
            (0xFF, 0x00, 0xFF) => "Magenta",
            (0xC0, 0x00, 0x00) => "Dark Red",
            (0xC0, 0xC0, 0x00) => "Dark Yellow",
            (0x00, 0xC0, 0x00) => "Dark Green",
            (0x00, 0xC0, 0xC0) => "Dark Cyan",
            (0x00, 0x00, 0xC0) => "Dark Blue",
            (0xC0, 0x00, 0xC0) => "Dark Magenta",
            (0x00, 0x00, 0x00) => "Black",
            (0xFF, 0xFF, 0xFF) => "White",
            _ => "Unknown",
        };
        
        println!("  {}: #{:02X}{:02X}{:02X} ({})", x, r, g, b, color_name);
    }
    
    // Calcular las transiciones de color según Piet
    println!("\nTransiciones de color (operaciones Piet):");
    
    // Tabla estándar de operaciones Piet
    // https://www.dangermouse.net/esoteric/piet.html
    // (hue_change, light_change) => operación
    fn get_op(hue: usize, light: usize) -> &'static str {
        match (hue, light) {
            // Lightness 0
            (0, 0) => "NOP",
            (1, 0) => "PUSH",
            (2, 0) => "POP",
            (3, 0) => "ADD",
            (4, 0) => "SUBTRACT",
            (5, 0) => "MULTIPLY",
            // Lightness 1
            (0, 1) => "DIVIDE",
            (1, 1) => "MOD",
            (2, 1) => "NOT",
            (3, 1) => "GREATER",
            (4, 1) => "POINTER",
            (5, 1) => "SWITCH",
            // Lightness 2
            (0, 2) => "DUPLICATE",
            (1, 2) => "ROLL",
            (2, 2) => "IN_NUMBER",
            (3, 2) => "IN_CHAR",
            (4, 2) => "OUT_NUMBER",
            (5, 2) => "OUT_CHAR",
            _ => "UNKNOWN",
        }
    }
    
    // Mapear colores a (hue, lightness)
    fn color_to_hl(r: u8, g: u8, b: u8) -> Option<(usize, usize)> {
        match (r, g, b) {
            // Light (lightness 0)
            (0xFF, 0xC0, 0xC0) => Some((0, 0)), // Light Red
            (0xFF, 0xFF, 0xC0) => Some((1, 0)), // Light Yellow
            (0xC0, 0xFF, 0xC0) => Some((2, 0)), // Light Green
            (0xC0, 0xFF, 0xFF) => Some((3, 0)), // Light Cyan
            (0xC0, 0xC0, 0xFF) => Some((4, 0)), // Light Blue
            (0xFF, 0xC0, 0xFF) => Some((5, 0)), // Light Magenta
            // Normal (lightness 1)
            (0xFF, 0x00, 0x00) => Some((0, 1)), // Red
            (0xFF, 0xFF, 0x00) => Some((1, 1)), // Yellow
            (0x00, 0xFF, 0x00) => Some((2, 1)), // Green
            (0x00, 0xFF, 0xFF) => Some((3, 1)), // Cyan
            (0x00, 0x00, 0xFF) => Some((4, 1)), // Blue
            (0xFF, 0x00, 0xFF) => Some((5, 1)), // Magenta
            // Dark (lightness 2)
            (0xC0, 0x00, 0x00) => Some((0, 2)), // Dark Red
            (0xC0, 0xC0, 0x00) => Some((1, 2)), // Dark Yellow
            (0x00, 0xC0, 0x00) => Some((2, 2)), // Dark Green
            (0x00, 0xC0, 0xC0) => Some((3, 2)), // Dark Cyan
            (0x00, 0x00, 0xC0) => Some((4, 2)), // Dark Blue
            (0xC0, 0x00, 0xC0) => Some((5, 2)), // Dark Magenta
            _ => None, // Black, white, or unknown
        }
    }
    
    let mut prev_hl: Option<(usize, usize)> = None;
    
    for x in 0..width {
        let offset = row_start + x * 3;
        let b = data[offset];
        let g = data[offset + 1];
        let r = data[offset + 2];
        
        if let Some((hue, light)) = color_to_hl(r, g, b) {
            if let Some((prev_hue, prev_light)) = prev_hl {
                let hue_change = (hue + 6 - prev_hue) % 6;
                let light_change = (light + 3 - prev_light) % 3;
                let op = get_op(hue_change, light_change);
                println!("  {} -> {}: hue_diff={}, light_diff={} => {}", 
                    x-1, x, hue_change, light_change, op);
            }
            prev_hl = Some((hue, light));
        } else {
            println!("  {}: BLACK/WHITE (halt or boundary)", x);
            prev_hl = None;
        }
    }
}
