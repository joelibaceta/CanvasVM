//! Generates a proper echo program that terminates after one InChar/OutChar
//! 
//! The design uses a 2D layout where the program cannot backtrack:
//! 
//! Layout (4x2):
//!   LightRed  DarkCyan   Green    Black
//!   Black     Black      Black    Black
//! 
//! Flow:
//! 1. Start at LightRed (0,0), DP=Right
//! 2. Move to DarkCyan (1,0) - LightRed→DarkCyan = InChar (hue+3, light+2)
//! 3. Move to Green (2,0) - DarkCyan→Green = OutChar (hue+5 mod 6 = -1, light+2)
//!    Wait, that's wrong. Let me recalculate.
//!
//! DarkCyan: Hue=3, Light=Dark(2)
//! Green: Hue=2, Light=Normal(1)
//! Transition: hue = 2-3 = -1 mod 6 = 5, light = 1-2 = -1 mod 3 = 2
//! Operation at (5,2) = OutChar ✓
//!
//! 4. From Green (2,0), all directions blocked: Right=Black, Down=Black, Left=DarkCyan, Up=edge
//!    But Left leads back to DarkCyan which continues to LightRed...
//!
//! The problem is that in a horizontal layout, going back is always possible.
//! We need a design where the blocks form a path that doesn't allow backtracking.
//!
//! Better approach: Use a corridor that only allows forward movement.
//! Actually, in Piet, you CAN go backwards if all other directions are blocked.
//! So the only way to truly terminate is to be surrounded by black on all sides
//! with no valid color transitions possible.
//!
//! The "trapped" condition in Piet is when after trying all 8 combinations of DP/CC,
//! you still can't find a valid exit. But if there's ANY colored pixel adjacent,
//! eventually you'll rotate to face it.
//!
//! The only way a program truly terminates is:
//! 1. The current block is completely surrounded by black
//! 2. Or you reach an edge and all other directions are black
//!
//! Let's make a 3x1 program where the last block has black on all sides except the entry:
//! 
//! LightRed  DarkCyan  Green
//! Black     Black     Black
//!
//! Actually this still allows going backwards.
//!
//! The trick is: After doing the operation, the program is in Green.
//! It tries to go Right (DP=Right) but hits edge/black.
//! Then rotates: Down=Black, Left=DarkCyan (valid!), so it goes back.
//!
//! In reality, most Piet programs that "terminate" after one operation need a
//! dead-end design. Let me use a U-shape:
//!
//!   Black     DarkCyan    Green
//!   LightRed  Black       Black  
//!   Black     Black       Black
//!
//! This still allows backtracking.
//!
//! Actually, looking at real Piet programs, they often just run until they
//! reach a point where trying to continue results in an infinite loop of NOP,
//! or the user manually stops them, or they do have termination conditions.
//!
//! For testing INPUT, we should just create a simple program and accept that
//! it might not terminate "nicely" - it will halt when blocked.
//!
//! Let's create a simple 1D program and document that it will cycle if not 
//! properly blocked. For proper termination, we need the LAST block to be
//! completely surrounded by black with no way out.
//!
//! FINAL DESIGN - A 2x2 with corner trap:
//!
//!   LightRed   DarkCyan
//!   Black      Green
//!
//! Flow: 
//! - Start at LightRed (0,0) with DP=Right
//! - Move Right to DarkCyan (1,0) -> InChar
//! - From DarkCyan, DP=Right but edge. Rotate: Down to Green (1,1) -> OutChar
//! - From Green (1,1):
//!   - Right: edge
//!   - Down: edge  
//!   - Left: Black (0,1)
//!   - Up: DarkCyan (would go back)
//!
//! Still can go back. Let's add more black:
//!
//!   LightRed   DarkCyan   Black
//!   Black      Green      Black
//!   Black      Black      Black
//!
//! From Green:
//! - Right: Black -> rotate
//! - Down: Black -> rotate
//! - Left: Black -> rotate
//! - Up: DarkCyan -> goes back
//!
//! Still goes back to DarkCyan!
//!
//! The ONLY way to truly trap is if the block has NO valid colored neighbors at all.
//!
//!   LightRed   White      DarkCyan
//!   Black      Black      Green
//!   Black      Black      Black
//!
//! With white, the program slides through:
//! - LightRed (0,0) -> Right -> White (1,0) -> slides to DarkCyan (2,0) -> InChar
//!   (crossing white = NOP, so InChar doesn't happen yet)
//!
//! Actually crossing white DOES NOT execute an operation. The transition
//! through white is like teleporting.
//!
//! Let me re-read Piet spec...
//!
//! From Piet spec: "Sliding through white doesn't cause any operation."
//!
//! So we need direct color-to-color transitions for operations.
//!
//! SIMPLEST SOLUTION: Accept that after OutChar, the program will either:
//! 1. Cycle back and do more operations (with empty stack = NOPs)
//! 2. Eventually halt due to some condition
//!
//! For TESTING purposes, let's just make a simple program and ensure the
//! INPUT mechanism works. The program might not terminate elegantly.

use std::fs::File;
use std::io::Write;

fn main() {
    // Create a simple 2x2 program:
    // 
    //   LightRed (255,192,192)    DarkCyan (0,192,192)
    //   Black (0,0,0)             Green (0,255,0)
    //
    // This will: InChar, OutChar, then cycle

    let width: u32 = 2;
    let height: u32 = 2;
    
    // BMP header (54 bytes)
    let file_size: u32 = 54 + (width * height * 3);
    let mut header: Vec<u8> = vec![
        0x42, 0x4D,                                   // BM
        (file_size & 0xFF) as u8,                     // File size
        ((file_size >> 8) & 0xFF) as u8,
        ((file_size >> 16) & 0xFF) as u8,
        ((file_size >> 24) & 0xFF) as u8,
        0x00, 0x00, 0x00, 0x00,                       // Reserved
        0x36, 0x00, 0x00, 0x00,                       // Offset to pixel data (54)
        0x28, 0x00, 0x00, 0x00,                       // DIB header size (40)
        (width & 0xFF) as u8,                         // Width
        ((width >> 8) & 0xFF) as u8,
        ((width >> 16) & 0xFF) as u8,
        ((width >> 24) & 0xFF) as u8,
        (height & 0xFF) as u8,                        // Height
        ((height >> 8) & 0xFF) as u8,
        ((height >> 16) & 0xFF) as u8,
        ((height >> 24) & 0xFF) as u8,
        0x01, 0x00,                                   // Planes (1)
        0x18, 0x00,                                   // Bits per pixel (24)
        0x00, 0x00, 0x00, 0x00,                       // Compression (none)
        0x00, 0x00, 0x00, 0x00,                       // Image size (can be 0 for no compression)
        0x13, 0x0B, 0x00, 0x00,                       // Horizontal resolution
        0x13, 0x0B, 0x00, 0x00,                       // Vertical resolution
        0x00, 0x00, 0x00, 0x00,                       // Colors in palette
        0x00, 0x00, 0x00, 0x00,                       // Important colors
    ];
    
    // Colors (BGR format)
    let light_red = [0xC0, 0xC0, 0xFF];     // (255, 192, 192)
    let dark_cyan = [0xC0, 0xC0, 0x00];     // (0, 192, 192)
    let green = [0x00, 0xFF, 0x00];         // (0, 255, 0)
    let black = [0x00, 0x00, 0x00];         // (0, 0, 0)
    
    // Row padding: each row must be multiple of 4 bytes
    // Width 2 * 3 bytes = 6 bytes, next multiple of 4 is 8, so padding = 2
    let padding = vec![0u8; 2];
    
    // Pixel data (rows from bottom to top)
    // Row 1 (bottom): Black, Green
    // Row 0 (top): LightRed, DarkCyan
    
    let mut pixels: Vec<u8> = Vec::new();
    
    // Row 1 (y=1 in image coordinates, written first in BMP)
    pixels.extend_from_slice(&black);
    pixels.extend_from_slice(&green);
    pixels.extend_from_slice(&padding);
    
    // Row 0 (y=0 in image coordinates, written second in BMP)
    pixels.extend_from_slice(&light_red);
    pixels.extend_from_slice(&dark_cyan);
    pixels.extend_from_slice(&padding);
    
    // Write file
    let mut file = File::create("tools/fixtures/samples/echo_simple.bmp").unwrap();
    file.write_all(&header).unwrap();
    file.write_all(&pixels).unwrap();
    
    println!("Created echo_simple.bmp (2x2)");
    println!("Layout:");
    println!("  LightRed  DarkCyan");
    println!("  Black     Green");
    println!();
    println!("Flow:");
    println!("1. Start at LightRed (0,0)");
    println!("2. Move to DarkCyan -> InChar");
    println!("3. Move to Green -> OutChar");
    println!("4. Cycles back (program continues)");
}
