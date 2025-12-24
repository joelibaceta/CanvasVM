//! Generates a Piet program that reads 4 chars then prints them IN ORDER
//! 
//! Stack is LIFO, so we need to use Roll operations to reverse the order.
//! 
//! Operations:
//! 1-4. InChar x4 → stack = [A, B, C, D] (D on top)
//! 5. Push 4, 6. Push 1, 7. Roll → stack = [B, C, D, A]
//! 8. OutChar → prints A
//! 9. Push 3, 10. Push 1, 11. Roll → [C, D, B]
//! 12. OutChar → prints B
//! 13. Push 2, 14. Push 1, 15. Roll → [D, C]
//! 16. OutChar → prints C
//! 17. OutChar → prints D
//! 18. HALT
//!
//! Color calculations:
//! - InChar: hue+3, light+2
//! - Push: hue+1, light+0  (block size = value to push)
//! - Roll: hue+1, light+2
//! - OutChar: hue+5, light+2

use std::fs::File;
use std::io::Write;

// Hue: 0=Red, 1=Yellow, 2=Green, 3=Cyan, 4=Blue, 5=Magenta
// Light: 0=Light, 1=Normal, 2=Dark

fn color_rgb(hue: u8, light: u8) -> [u8; 3] {
    // Returns BGR for BMP format
    match (hue % 6, light % 3) {
        (0, 0) => [0xC0, 0xC0, 0xFF], // LightRed
        (0, 1) => [0x00, 0x00, 0xFF], // Red
        (0, 2) => [0x00, 0x00, 0xC0], // DarkRed
        (1, 0) => [0xC0, 0xFF, 0xFF], // LightYellow
        (1, 1) => [0x00, 0xFF, 0xFF], // Yellow
        (1, 2) => [0x00, 0xC0, 0xC0], // DarkYellow
        (2, 0) => [0xC0, 0xFF, 0xC0], // LightGreen
        (2, 1) => [0x00, 0xFF, 0x00], // Green
        (2, 2) => [0x00, 0xC0, 0x00], // DarkGreen
        (3, 0) => [0xFF, 0xFF, 0xC0], // LightCyan
        (3, 1) => [0xFF, 0xFF, 0x00], // Cyan
        (3, 2) => [0xC0, 0xC0, 0x00], // DarkCyan
        (4, 0) => [0xFF, 0xC0, 0xC0], // LightBlue
        (4, 1) => [0xFF, 0x00, 0x00], // Blue
        (4, 2) => [0xC0, 0x00, 0x00], // DarkBlue
        (5, 0) => [0xFF, 0xC0, 0xFF], // LightMagenta
        (5, 1) => [0xFF, 0x00, 0xFF], // Magenta
        (5, 2) => [0xC0, 0x00, 0xC0], // DarkMagenta
        _ => [0x00, 0x00, 0x00],      // Black
    }
}

fn color_name(hue: u8, light: u8) -> &'static str {
    match (hue % 6, light % 3) {
        (0, 0) => "LightRed",
        (0, 1) => "Red",
        (0, 2) => "DarkRed",
        (1, 0) => "LightYellow",
        (1, 1) => "Yellow",
        (1, 2) => "DarkYellow",
        (2, 0) => "LightGreen",
        (2, 1) => "Green",
        (2, 2) => "DarkGreen",
        (3, 0) => "LightCyan",
        (3, 1) => "Cyan",
        (3, 2) => "DarkCyan",
        (4, 0) => "LightBlue",
        (4, 1) => "Blue",
        (4, 2) => "DarkBlue",
        (5, 0) => "LightMagenta",
        (5, 1) => "Magenta",
        (5, 2) => "DarkMagenta",
        _ => "Black",
    }
}

fn main() {
    // Build the sequence of (hue, light, block_size)
    // block_size matters for Push operations
    let mut blocks: Vec<(u8, u8, u32)> = Vec::new();
    
    let mut hue: u8 = 0;
    let mut light: u8 = 0;
    
    // Helper to add transition
    let mut add_op = |blocks: &mut Vec<(u8, u8, u32)>, h: &mut u8, l: &mut u8, dh: u8, dl: u8, size: u32, op: &str| {
        let old_h = *h;
        let old_l = *l;
        *h = (*h + dh) % 6;
        *l = (*l + dl) % 3;
        println!("  {}({},{}) --{}--> {}({},{}) [size={}]", 
                 color_name(old_h, old_l), old_h, old_l,
                 op,
                 color_name(*h, *l), *h, *l, size);
        blocks.push((*h, *l, size));
    };
    
    // Start block
    blocks.push((0, 0, 1)); // LightRed, size 1
    println!("Start: LightRed(0,0)");
    
    // 4x InChar (+3, +2)
    add_op(&mut blocks, &mut hue, &mut light, 3, 2, 1, "InChar");
    add_op(&mut blocks, &mut hue, &mut light, 3, 2, 1, "InChar");
    add_op(&mut blocks, &mut hue, &mut light, 3, 2, 1, "InChar");
    add_op(&mut blocks, &mut hue, &mut light, 3, 2, 1, "InChar");
    
    // Push 4 (+1, +0) - block size 4
    add_op(&mut blocks, &mut hue, &mut light, 1, 0, 4, "Push4");
    // Push 1 (+1, +0) - block size 1
    add_op(&mut blocks, &mut hue, &mut light, 1, 0, 1, "Push1");
    // Roll (+1, +2)
    add_op(&mut blocks, &mut hue, &mut light, 1, 2, 1, "Roll");
    // OutChar (+5, +2)
    add_op(&mut blocks, &mut hue, &mut light, 5, 2, 1, "OutChar");
    
    // Push 3, Push 1, Roll, OutChar
    add_op(&mut blocks, &mut hue, &mut light, 1, 0, 3, "Push3");
    add_op(&mut blocks, &mut hue, &mut light, 1, 0, 1, "Push1");
    add_op(&mut blocks, &mut hue, &mut light, 1, 2, 1, "Roll");
    add_op(&mut blocks, &mut hue, &mut light, 5, 2, 1, "OutChar");
    
    // Push 2, Push 1, Roll, OutChar
    add_op(&mut blocks, &mut hue, &mut light, 1, 0, 2, "Push2");
    add_op(&mut blocks, &mut hue, &mut light, 1, 0, 1, "Push1");
    add_op(&mut blocks, &mut hue, &mut light, 1, 2, 1, "Roll");
    add_op(&mut blocks, &mut hue, &mut light, 5, 2, 1, "OutChar");
    
    // Final OutChar
    add_op(&mut blocks, &mut hue, &mut light, 5, 2, 1, "OutChar");
    
    // Calculate total width needed
    let total_pixels: u32 = blocks.iter().map(|(_, _, s)| s).sum::<u32>() + 1; // +1 for black terminator
    let width = total_pixels;
    let height: u32 = 2;
    
    println!("\nTotal pixels in top row: {}", total_pixels);
    println!("Image size: {}x{}", width, height);
    
    // Row stride
    let row_bytes = width * 3;
    let padding = (4 - (row_bytes % 4)) % 4;
    let stride = row_bytes + padding;
    let pixel_data_size = stride * height;
    let file_size: u32 = 54 + pixel_data_size;
    
    // BMP header
    let header: Vec<u8> = vec![
        0x42, 0x4D,
        (file_size & 0xFF) as u8,
        ((file_size >> 8) & 0xFF) as u8,
        ((file_size >> 16) & 0xFF) as u8,
        ((file_size >> 24) & 0xFF) as u8,
        0x00, 0x00, 0x00, 0x00,
        0x36, 0x00, 0x00, 0x00,
        0x28, 0x00, 0x00, 0x00,
        (width & 0xFF) as u8,
        ((width >> 8) & 0xFF) as u8,
        ((width >> 16) & 0xFF) as u8,
        ((width >> 24) & 0xFF) as u8,
        (height & 0xFF) as u8,
        ((height >> 8) & 0xFF) as u8,
        ((height >> 16) & 0xFF) as u8,
        ((height >> 24) & 0xFF) as u8,
        0x01, 0x00,
        0x18, 0x00,
        0x00, 0x00, 0x00, 0x00,
        (pixel_data_size & 0xFF) as u8,
        ((pixel_data_size >> 8) & 0xFF) as u8,
        ((pixel_data_size >> 16) & 0xFF) as u8,
        ((pixel_data_size >> 24) & 0xFF) as u8,
        0x13, 0x0B, 0x00, 0x00,
        0x13, 0x0B, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00,
    ];
    
    let black = [0x00u8, 0x00, 0x00];
    
    // Build pixel rows
    let mut pixels: Vec<u8> = Vec::new();
    
    // Row 1 (bottom): all black
    for _ in 0..width {
        pixels.extend_from_slice(&black);
    }
    for _ in 0..padding {
        pixels.push(0);
    }
    
    // Row 0 (top): colored blocks
    let mut x = 0u32;
    for (h, l, size) in &blocks {
        let color = color_rgb(*h, *l);
        for _ in 0..*size {
            pixels.extend_from_slice(&color);
            x += 1;
        }
    }
    // Black terminator
    pixels.extend_from_slice(&black);
    x += 1;
    
    // Fill rest with black if needed
    while x < width {
        pixels.extend_from_slice(&black);
        x += 1;
    }
    for _ in 0..padding {
        pixels.push(0);
    }
    
    // Write file
    let mut file = File::create("tools/fixtures/samples/echo4_ordered.bmp").unwrap();
    file.write_all(&header).unwrap();
    file.write_all(&pixels).unwrap();
    
    println!("\n✓ Created echo4_ordered.bmp");
    println!("\nInput 'ABCD' → Output 'ABCD' (in same order!)");
}
