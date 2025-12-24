//! Genera una imagen Piet que lee 4 caracteres y los imprime
//! Compilar: rustc tools/generate_echo4.rs -o generate_echo4
//! Ejecutar: ./generate_echo4

use std::fs::File;
use std::io::Write;

// Colores Piet (6 tonos x 3 luminosidades)
const PIET_COLORS: [[u8; 3]; 18] = [
    // Light
    [0xFF, 0xC0, 0xC0], // light red
    [0xFF, 0xFF, 0xC0], // light yellow
    [0xC0, 0xFF, 0xC0], // light green
    [0xC0, 0xFF, 0xFF], // light cyan
    [0xC0, 0xC0, 0xFF], // light blue
    [0xFF, 0xC0, 0xFF], // light magenta
    // Normal
    [0xFF, 0x00, 0x00], // red
    [0xFF, 0xFF, 0x00], // yellow
    [0x00, 0xFF, 0x00], // green
    [0x00, 0xFF, 0xFF], // cyan
    [0x00, 0x00, 0xFF], // blue
    [0xFF, 0x00, 0xFF], // magenta
    // Dark
    [0xC0, 0x00, 0x00], // dark red
    [0xC0, 0xC0, 0x00], // dark yellow
    [0x00, 0xC0, 0x00], // dark green
    [0x00, 0xC0, 0xC0], // dark cyan
    [0x00, 0x00, 0xC0], // dark blue
    [0xC0, 0x00, 0xC0], // dark magenta
];

const BLACK: [u8; 3] = [0x00, 0x00, 0x00];

fn get_color(hue: usize, lightness: usize) -> [u8; 3] {
    let idx = (lightness % 3) * 6 + (hue % 6);
    PIET_COLORS[idx]
}

/// Escribe un archivo BMP de 24 bits (sin dependencias externas)
fn write_bmp(filename: &str, width: u32, height: u32, pixels: &[[u8; 3]]) -> std::io::Result<()> {
    let mut file = File::create(filename)?;
    
    // BMP row padding (each row must be multiple of 4 bytes)
    let row_size = ((width * 3 + 3) / 4) * 4;
    let pixel_data_size = row_size * height;
    let file_size = 54 + pixel_data_size;
    
    // BMP Header (14 bytes)
    file.write_all(b"BM")?;                           // Signature
    file.write_all(&(file_size as u32).to_le_bytes())?; // File size
    file.write_all(&[0u8; 4])?;                       // Reserved
    file.write_all(&54u32.to_le_bytes())?;            // Pixel data offset
    
    // DIB Header (40 bytes - BITMAPINFOHEADER)
    file.write_all(&40u32.to_le_bytes())?;            // Header size
    file.write_all(&(width as i32).to_le_bytes())?;   // Width
    file.write_all(&(height as i32).to_le_bytes())?;  // Height (positive = bottom-up)
    file.write_all(&1u16.to_le_bytes())?;             // Color planes
    file.write_all(&24u16.to_le_bytes())?;            // Bits per pixel
    file.write_all(&0u32.to_le_bytes())?;             // Compression (none)
    file.write_all(&pixel_data_size.to_le_bytes())?;  // Image size
    file.write_all(&2835u32.to_le_bytes())?;          // X pixels per meter
    file.write_all(&2835u32.to_le_bytes())?;          // Y pixels per meter
    file.write_all(&0u32.to_le_bytes())?;             // Colors in color table
    file.write_all(&0u32.to_le_bytes())?;             // Important colors
    
    // Pixel data (bottom-up, BGR format)
    let padding = vec![0u8; (row_size - width * 3) as usize];
    for y in (0..height).rev() {
        for x in 0..width {
            let idx = (y * width + x) as usize;
            let [r, g, b] = pixels[idx];
            file.write_all(&[b, g, r])?; // BGR
        }
        file.write_all(&padding)?;
    }
    
    Ok(())
}

fn main() -> std::io::Result<()> {
    // Generar AMBAS imágenes
    
    // === ECHO1: 1 InChar + 1 OutChar + Negro ===
    println!("=== Generando echo1 (versión simple) ===");
    let mut pixels1: Vec<[u8; 3]> = Vec::new();
    let mut hue = 0usize;
    let mut light = 0usize;
    
    // Color inicial (Light Red)
    pixels1.push(get_color(hue, light));
    
    // InChar: hue+3, light+2
    hue = (hue + 3) % 6;
    light = (light + 2) % 3;
    pixels1.push(get_color(hue, light));
    
    // OutChar: hue+5, light+2
    hue = (hue + 5) % 6;
    light = (light + 2) % 3;
    pixels1.push(get_color(hue, light));
    
    // Negro para terminar
    pixels1.push(BLACK);
    
    let width1 = pixels1.len() as u32;
    write_bmp("tools/fixtures/samples/echo1.bmp", width1, 1, &pixels1)?;
    println!("✓ Generado: tools/fixtures/samples/echo1.bmp ({}x1)", width1);
    
    println!("\nSecuencia echo1:");
    for (i, p) in pixels1.iter().enumerate() {
        println!("  {}: #{:02X}{:02X}{:02X}", i, p[0], p[1], p[2]);
    }
    
    // Versión escalada
    let scale = 10u32;
    let mut scaled1: Vec<[u8; 3]> = Vec::new();
    for _ in 0..scale {
        for pixel in &pixels1 {
            for _ in 0..scale {
                scaled1.push(*pixel);
            }
        }
    }
    write_bmp("tools/fixtures/samples/echo1_10x.bmp", width1 * scale, scale, &scaled1)?;
    println!("✓ Generado: tools/fixtures/samples/echo1_10x.bmp");
    
    // === ECHO4: 4x InChar + 4x OutChar + Negro ===
    println!("\n=== Generando echo4 (4 caracteres) ===");
    let mut pixels4: Vec<[u8; 3]> = Vec::new();
    hue = 0;
    light = 0;
    
    // Color inicial
    pixels4.push(get_color(hue, light));
    
    for i in 0..4 {
        // InChar: hue+3, light+2
        hue = (hue + 3) % 6;
        light = (light + 2) % 3;
        pixels4.push(get_color(hue, light));
        println!("  InChar {}: hue={}, light={} -> #{:02X}{:02X}{:02X}", 
                 i+1, hue, light, 
                 pixels4.last().unwrap()[0], 
                 pixels4.last().unwrap()[1], 
                 pixels4.last().unwrap()[2]);
        
        // OutChar: hue+5, light+2
        hue = (hue + 5) % 6;
        light = (light + 2) % 3;
        pixels4.push(get_color(hue, light));
        println!("  OutChar {}: hue={}, light={} -> #{:02X}{:02X}{:02X}", 
                 i+1, hue, light,
                 pixels4.last().unwrap()[0], 
                 pixels4.last().unwrap()[1], 
                 pixels4.last().unwrap()[2]);
    }
    
    // Negro para terminar
    pixels4.push(BLACK);
    
    let width4 = pixels4.len() as u32;
    write_bmp("tools/fixtures/samples/echo4.bmp", width4, 1, &pixels4)?;
    println!("✓ Generado: tools/fixtures/samples/echo4.bmp ({}x1)", width4);
    
    println!("\nSecuencia echo4:");
    for (i, p) in pixels4.iter().enumerate() {
        println!("  {}: #{:02X}{:02X}{:02X}", i, p[0], p[1], p[2]);
    }
    
    // Versión escalada
    let mut scaled4: Vec<[u8; 3]> = Vec::new();
    for _ in 0..scale {
        for pixel in &pixels4 {
            for _ in 0..scale {
                scaled4.push(*pixel);
            }
        }
    }
    write_bmp("tools/fixtures/samples/echo4_10x.bmp", width4 * scale, scale, &scaled4)?;
    println!("✓ Generado: tools/fixtures/samples/echo4_10x.bmp");
    
    Ok(())
}
