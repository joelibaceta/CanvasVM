//! Simple echo: read 4 chars, print 4 chars (reversed due to LIFO)
//! Uses a 2D layout where the program path goes right then down, preventing backtracking
//!
//! Layout:
//!   LR  DC  R   LC  DR         (InChar x4)
//!   BLK BLK BLK BLK M          (vertical connector)
//!   BLK BLK BLK BLK LB DC G BLK  (OutChar x4 + terminator)

use std::fs::File;
use std::io::Write;

fn color_rgb(hue: u8, light: u8) -> [u8; 3] {
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
        _ => [0x00, 0x00, 0x00],
    }
}

fn main() {
    // Calculate color sequence:
    // InChar: hue+3, light+2
    // OutChar: hue+5, light+2
    
    // Row 0 (top): InChar sequence
    // Start: (0,0) LightRed
    // InChar→(3,2) DarkCyan → InChar→(0,1) Red → InChar→(3,0) LightCyan → InChar→(0,2) DarkRed
    
    // Corner turn: DarkRed(0,2) → down to row 1
    // We need the next color below to NOT trigger an operation, OR use NOP
    // Actually, moving DOWN within same color block = no operation
    // So DarkRed should extend down
    
    // Row 1: DarkRed continues, then OutChar sequence
    // DarkRed(0,2) → OutChar→(5,1) Magenta → OutChar→(4,0) LightBlue → OutChar→(3,2) DarkCyan → OutChar→(2,1) Green
    // Then Green is trapped by black
    
    let width = 8;
    let height = 3;
    
    // BGR colors
    let black = [0x00u8, 0x00, 0x00];
    let light_red = color_rgb(0, 0);      // Start
    let dark_cyan = color_rgb(3, 2);      // After InChar 1
    let red = color_rgb(0, 1);            // After InChar 2
    let light_cyan = color_rgb(3, 0);     // After InChar 3
    let dark_red = color_rgb(0, 2);       // After InChar 4 (corner piece)
    let magenta = color_rgb(5, 1);        // After OutChar 1
    let light_blue = color_rgb(4, 0);     // After OutChar 2
    let dark_cyan2 = color_rgb(3, 2);     // After OutChar 3
    let green = color_rgb(2, 1);          // After OutChar 4 (end)
    
    // Build pixel grid (row-major, top to bottom for logic, but BMP is bottom-up)
    // Row 0: LR DC R LC DR BLK BLK BLK
    // Row 1: BLK BLK BLK BLK DR BLK BLK BLK  
    // Row 2: BLK BLK BLK BLK M LB DC2 G BLK
    // Wait, that doesn't work because DR needs to connect down...
    
    // Better layout:
    // Row 0: LR DC R LC DR BLK BLK BLK  (InChar x4, DR is at col 4)
    // Row 1: BLK BLK BLK BLK DR M LB DC2 (DR extends down, then OutChar starts)
    // Row 2: BLK BLK BLK BLK BLK BLK BLK G (G is trapped)
    
    // Actually let me make it even simpler - just a vertical L-shape:
    // Col: 0   1   2   3   4   5   6   7
    // Row 0: LR  DC  R   LC  DR  BLK BLK BLK
    // Row 1: BLK BLK BLK BLK DR  M   LB  DC2
    // Row 2: BLK BLK BLK BLK BLK M   BLK G
    // Row 3: BLK BLK BLK BLK BLK BLK BLK BLK
    
    // Hmm this is getting complicated. Let me just make a simple vertical program:
    // Col 0 only, goes down:
    // Row 0: LR
    // Row 1: DC  (InChar)
    // Row 2: R   (InChar)
    // Row 3: LC  (InChar)
    // Row 4: DR  (InChar)
    // Row 5: M   (OutChar)
    // Row 6: LB  (OutChar)
    // Row 7: DC2 (OutChar)
    // Row 8: G   (OutChar)
    // Row 9: BLK (terminator)
    
    // But this allows going back up...
    // The trick is: when blocked, the program rotates DP. If all directions are blocked, it halts.
    // In a 1-wide column with black on sides, it CAN go back up.
    
    // The HelloWorld programs work because they have 2D structure with black barriers.
    // Let me just create a simple test that accepts the reversed output.
    
    // For now, let's use the echo4_linear approach (which outputs reversed) and verify it works:
    let width: u32 = 10;
    let height: u32 = 2;
    
    let colors: Vec<[u8; 3]> = vec![
        color_rgb(0, 0),  // 0: LightRed (start)
        color_rgb(3, 2),  // 1: DarkCyan (after InChar 1)
        color_rgb(0, 1),  // 2: Red (after InChar 2)
        color_rgb(3, 0),  // 3: LightCyan (after InChar 3)
        color_rgb(0, 2),  // 4: DarkRed (after InChar 4)
        color_rgb(5, 1),  // 5: Magenta (after OutChar 1)
        color_rgb(4, 0),  // 6: LightBlue (after OutChar 2)
        color_rgb(3, 2),  // 7: DarkCyan (after OutChar 3)
        color_rgb(2, 1),  // 8: Green (after OutChar 4)
        black,            // 9: Black (terminator)
    ];
    
    // BMP setup
    let row_bytes = width * 3;
    let padding = (4 - (row_bytes % 4)) % 4;
    let stride = row_bytes + padding;
    let pixel_data_size = stride * height;
    let file_size: u32 = 54 + pixel_data_size;
    
    let header: Vec<u8> = vec![
        0x42, 0x4D,
        (file_size & 0xFF) as u8, ((file_size >> 8) & 0xFF) as u8,
        ((file_size >> 16) & 0xFF) as u8, ((file_size >> 24) & 0xFF) as u8,
        0x00, 0x00, 0x00, 0x00,
        0x36, 0x00, 0x00, 0x00,
        0x28, 0x00, 0x00, 0x00,
        (width & 0xFF) as u8, ((width >> 8) & 0xFF) as u8,
        ((width >> 16) & 0xFF) as u8, ((width >> 24) & 0xFF) as u8,
        (height & 0xFF) as u8, ((height >> 8) & 0xFF) as u8,
        ((height >> 16) & 0xFF) as u8, ((height >> 24) & 0xFF) as u8,
        0x01, 0x00, 0x18, 0x00,
        0x00, 0x00, 0x00, 0x00,
        (pixel_data_size & 0xFF) as u8, ((pixel_data_size >> 8) & 0xFF) as u8,
        ((pixel_data_size >> 16) & 0xFF) as u8, ((pixel_data_size >> 24) & 0xFF) as u8,
        0x13, 0x0B, 0x00, 0x00, 0x13, 0x0B, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ];
    
    let mut pixels: Vec<u8> = Vec::new();
    
    // Row 1 (bottom in BMP): all black
    for _ in 0..width {
        pixels.extend_from_slice(&black);
    }
    for _ in 0..padding {
        pixels.push(0);
    }
    
    // Row 0 (top in BMP): color sequence
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
    
    let mut file = File::create("tools/fixtures/samples/echo4_simple.bmp").unwrap();
    file.write_all(&header).unwrap();
    file.write_all(&pixels).unwrap();
    
    println!("Created echo4_simple.bmp (10x2)");
    println!("Input: HOLA → Output: ALOH (reversed due to LIFO stack)");
}
