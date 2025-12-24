//! Generates a Piet program that reads 4 chars then prints them
//! 
//! Operations needed:
//! - 4x InChar (hue+3, light+2)
//! - 4x OutChar (hue+5, light+2)
//! 
//! Color sequence calculated:
//! 1. LightRed(0,0) → InChar → DarkCyan(3,2)
//! 2. DarkCyan(3,2) → InChar → Red(0,1)        [hue: 3+3=6%6=0, light: 2+2=4%3=1]
//! 3. Red(0,1) → InChar → LightCyan(3,0)       [hue: 0+3=3, light: 1+2=3%3=0]
//! 4. LightCyan(3,0) → InChar → DarkRed(0,2)   [hue: 3+3=6%6=0, light: 0+2=2]
//! 5. DarkRed(0,2) → OutChar → Magenta(5,1)    [hue: 0+5=5, light: 2+2=4%3=1]
//! 6. Magenta(5,1) → OutChar → LightBlue(4,0)  [hue: 5+5=10%6=4, light: 1+2=3%3=0]
//! 7. LightBlue(4,0) → OutChar → DarkCyan(3,2) [hue: 4+5=9%6=3, light: 0+2=2]
//! 8. DarkCyan(3,2) → OutChar → Green(2,1)     [hue: 3+5=8%6=2, light: 2+2=4%3=1]
//! 9. Green → trapped by black → HALT
//!
//! Note: Stack is LIFO, so chars are printed in reverse order (last in, first out)
//! Input: ABCD → Output: DCBA

use std::fs::File;
use std::io::Write;

fn main() {
    // 9 color blocks + 1 black terminator in a row
    // Plus a row of black below to ensure termination
    let width: u32 = 10;
    let height: u32 = 2;
    
    // Colors (BGR format for BMP)
    let colors: Vec<[u8; 3]> = vec![
        [0xC0, 0xC0, 0xFF], // 0: LightRed (255,192,192)
        [0xC0, 0xC0, 0x00], // 1: DarkCyan (0,192,192)
        [0x00, 0x00, 0xFF], // 2: Red (255,0,0)
        [0xFF, 0xFF, 0xC0], // 3: LightCyan (192,255,255)
        [0x00, 0x00, 0xC0], // 4: DarkRed (192,0,0)
        [0xFF, 0x00, 0xFF], // 5: Magenta (255,0,255)
        [0xFF, 0xC0, 0xC0], // 6: LightBlue (192,192,255)
        [0xC0, 0xC0, 0x00], // 7: DarkCyan again (0,192,192)
        [0x00, 0xFF, 0x00], // 8: Green (0,255,0)
        [0x00, 0x00, 0x00], // 9: Black (terminator)
    ];
    
    let black = [0x00u8, 0x00, 0x00];
    
    // Row stride (padded to 4 bytes)
    let row_bytes = width * 3;
    let padding = (4 - (row_bytes % 4)) % 4;
    let stride = row_bytes + padding;
    
    // BMP header
    let pixel_data_size = stride * height;
    let file_size: u32 = 54 + pixel_data_size;
    
    let header: Vec<u8> = vec![
        0x42, 0x4D,                                   // BM
        (file_size & 0xFF) as u8,
        ((file_size >> 8) & 0xFF) as u8,
        ((file_size >> 16) & 0xFF) as u8,
        ((file_size >> 24) & 0xFF) as u8,
        0x00, 0x00, 0x00, 0x00,                       // Reserved
        0x36, 0x00, 0x00, 0x00,                       // Offset to pixel data (54)
        0x28, 0x00, 0x00, 0x00,                       // DIB header size (40)
        (width & 0xFF) as u8,
        ((width >> 8) & 0xFF) as u8,
        ((width >> 16) & 0xFF) as u8,
        ((width >> 24) & 0xFF) as u8,
        (height & 0xFF) as u8,
        ((height >> 8) & 0xFF) as u8,
        ((height >> 16) & 0xFF) as u8,
        ((height >> 24) & 0xFF) as u8,
        0x01, 0x00,                                   // Planes (1)
        0x18, 0x00,                                   // Bits per pixel (24)
        0x00, 0x00, 0x00, 0x00,                       // Compression (none)
        (pixel_data_size & 0xFF) as u8,
        ((pixel_data_size >> 8) & 0xFF) as u8,
        ((pixel_data_size >> 16) & 0xFF) as u8,
        ((pixel_data_size >> 24) & 0xFF) as u8,
        0x13, 0x0B, 0x00, 0x00,                       // Horizontal resolution
        0x13, 0x0B, 0x00, 0x00,                       // Vertical resolution
        0x00, 0x00, 0x00, 0x00,                       // Colors in palette
        0x00, 0x00, 0x00, 0x00,                       // Important colors
    ];
    
    // Pixel data (rows from bottom to top in BMP)
    let mut pixels: Vec<u8> = Vec::new();
    
    // Row 1 (y=1, bottom row in BMP file): all black
    for _ in 0..width {
        pixels.extend_from_slice(&black);
    }
    for _ in 0..padding {
        pixels.push(0);
    }
    
    // Row 0 (y=0, top row in BMP file): color sequence
    for i in 0..width as usize {
        if i < colors.len() {
            pixels.extend_from_slice(&colors[i]);
        } else {
            pixels.extend_from_slice(&black);
        }
    }
    for _ in 0..padding {
        pixels.push(0);
    }
    
    // Write file
    let mut file = File::create("tools/fixtures/samples/echo4_linear.bmp").unwrap();
    file.write_all(&header).unwrap();
    file.write_all(&pixels).unwrap();
    
    println!("Created echo4_linear.bmp ({}x{})", width, height);
    println!();
    println!("Layout (top row):");
    println!("  LightRed → DarkCyan → Red → LightCyan → DarkRed → Magenta → LightBlue → DarkCyan → Green → Black");
    println!();
    println!("Operations:");
    println!("  1. InChar (read char 1)");
    println!("  2. InChar (read char 2)");
    println!("  3. InChar (read char 3)");
    println!("  4. InChar (read char 4)");
    println!("  5. OutChar (print char 4) ← LIFO order");
    println!("  6. OutChar (print char 3)");
    println!("  7. OutChar (print char 2)");
    println!("  8. OutChar (print char 1)");
    println!("  9. HALT (trapped by black)");
    println!();
    println!("Note: Due to stack LIFO, input 'ABCD' outputs 'DCBA'");
}
