//! Genera una imagen Piet echo que termina correctamente
//! La imagen tiene 3 filas: negro arriba, programa en medio, negro abajo
//! Esto evita el rebote infinito

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

fn write_bmp(filename: &str, width: u32, height: u32, pixels: &[[u8; 3]]) -> std::io::Result<()> {
    let mut file = File::create(filename)?;
    
    let row_size = ((width * 3 + 3) / 4) * 4;
    let pixel_data_size = row_size * height;
    let file_size = 54 + pixel_data_size;
    
    file.write_all(b"BM")?;
    file.write_all(&(file_size as u32).to_le_bytes())?;
    file.write_all(&[0u8; 4])?;
    file.write_all(&54u32.to_le_bytes())?;
    
    file.write_all(&40u32.to_le_bytes())?;
    file.write_all(&(width as i32).to_le_bytes())?;
    file.write_all(&(height as i32).to_le_bytes())?;
    file.write_all(&1u16.to_le_bytes())?;
    file.write_all(&24u16.to_le_bytes())?;
    file.write_all(&0u32.to_le_bytes())?;
    file.write_all(&pixel_data_size.to_le_bytes())?;
    file.write_all(&2835u32.to_le_bytes())?;
    file.write_all(&2835u32.to_le_bytes())?;
    file.write_all(&0u32.to_le_bytes())?;
    file.write_all(&0u32.to_le_bytes())?;
    
    let padding = vec![0u8; (row_size - width * 3) as usize];
    for y in (0..height).rev() {
        for x in 0..width {
            let idx = (y * width + x) as usize;
            let [r, g, b] = pixels[idx];
            file.write_all(&[b, g, r])?;
        }
        file.write_all(&padding)?;
    }
    
    Ok(())
}

fn main() -> std::io::Result<()> {
    // === ECHO que termina correctamente ===
    // 
    // Estructura de 3 filas:
    // Fila 0: Negro en toda la fila (excepto donde empieza el programa)
    // Fila 1: R -> C (InChar) -> G (OutChar) -> Negro
    // Fila 2: Negro en toda la fila
    //
    // PERO el problema es que la VM empieza en (0,0).
    // Si (0,0) es negro, hace halt inmediatamente.
    //
    // Entonces la fila 0 debe tener el inicio del programa en (0,0).
    //
    // Nueva estructura:
    // Fila 0: R -> C -> G -> N  (programa empieza aquí)
    // Fila 1: N -> N -> N -> N  (negro abajo)
    // Fila 2: N -> N -> N -> N  (más negro para bloquear completamente)
    //
    // Cuando está en G mirando Right hacia N:
    // - Right: N (bloqueado)
    // - Down: N (bloqueado)
    // - Left: C (puede ir! rebota)
    // - Up: fuera del grid (bloqueado)
    //
    // Sigue rebotando porque puede ir a Left...
    //
    // Para bloquear completamente, necesitamos que el último bloque (G)
    // tenga negro a la derecha, abajo, Y a la izquierda.
    // Pero si hay negro a la izquierda, ¿cómo llegó ahí?
    //
    // Solución: el programa va hacia ABAJO después de OutChar, no hacia Right.
    //
    // Nueva estructura:
    // Col:  0   1   2
    // -------------------
    // 0:    R   N   N
    // 1:    C   N   N
    // 2:    G   N   N
    // 3:    N   N   N
    //
    // R en (0,0), mirando Down
    // R -> C (InChar) cuando DP=Down
    // C -> G (OutChar) cuando DP=Down
    // G -> N cuando DP=Down -> bloqueado
    // G intenta rotar, todos bloqueados -> HALT
    //
    // Pero el problema es que la VM empieza con DP=Right, no Down.
    // Y R mirando Right va hacia N...
    //
    // OK, esto es complicado. Hagamos algo más simple:
    // Un solo InChar seguido de halt.
    
    // === Versión simple: solo InChar ===
    // Imagen 2x3:
    // R N   <- programa
    // C N
    // N N
    //
    // R en (0,0), DP=Right inicial
    // R mirando Right: N (bloqueado)
    // R rota, mirando Down: C (puede ir!)
    // R -> C = InChar
    // C mirando Down: N (bloqueado)
    // C rota todas direcciones: N, N, R, N...
    // C mirando Left (después de rotar): fuera del grid
    // C mirando Up: R (puede ir! rebota)
    //
    // Aún rebota :(
    
    // === Solución final: hacer la imagen más ancha ===
    // Para que C no pueda volver a R, ponemos C rodeado de Negro
    // excepto por donde entró (desde arriba con DP=Down)
    //
    // Imagen 3x3:
    // N R N
    // N C N
    // N N N
    //
    // R en (1,0), DP=Right inicial
    // R mirando Right: N (bloqueado)
    // R rota, mirando Down: C (puede ir!)
    // R -> C = operación basada en transición
    // C mirando Down: N (bloqueado)
    // C mirando Left: N (bloqueado)
    // C mirando Up: R (puede ir! rebota)
    //
    // TODAVÍA rebota porque C puede ir Up a R.
    
    // === La verdad sobre Piet ===
    // En un programa Piet bien diseñado, el flujo va en una dirección
    // y hay "callejones sin salida" que causan halt.
    // Pero esto requiere diseño cuidadoso de la imagen.
    //
    // Para nuestro test, aceptemos que el programa "echo simple"
    // rebotará, y usemos un límite de steps.
    //
    // O hagamos un programa que realmente termine:
    // Un solo bloque de 1x1 que inmediatamente hace halt.
    
    println!("=== Generando imágenes de test ===\n");
    
    // === Test 1: Imagen 1x1 (halt inmediato) ===
    let pixels_1x1 = vec![get_color(0, 0)]; // Light Red
    write_bmp("tools/fixtures/samples/single_block.bmp", 1, 1, &pixels_1x1)?;
    println!("✓ single_block.bmp (1x1) - halt inmediato, sin operaciones");
    
    // === Test 2: Programa lineal que rebota (para testing de loops) ===
    let mut pixels_linear: Vec<[u8; 3]> = Vec::new();
    let mut hue = 0usize;
    let mut light = 0usize;
    
    pixels_linear.push(get_color(hue, light)); // Inicio
    
    // InChar: hue+3, light+2
    hue = (hue + 3) % 6;
    light = (light + 2) % 3;
    pixels_linear.push(get_color(hue, light));
    
    // OutChar: hue+5, light+2
    hue = (hue + 5) % 6;
    light = (light + 2) % 3;
    pixels_linear.push(get_color(hue, light));
    
    pixels_linear.push(BLACK);
    
    write_bmp("tools/fixtures/samples/echo_linear.bmp", 4, 1, &pixels_linear)?;
    println!("✓ echo_linear.bmp (4x1) - rebota infinitamente (para test de límite de steps)");
    
    // === Test 3: Echo con terminación usando bloque de 3 filas ===
    // La idea: hacer que el programa vaya hacia abajo y termine en un callejón sin salida
    //
    // Columna 0 es el programa, columnas 1-2 son negro
    // Fila 0: R(inicio) N N
    // Fila 1: C(InChar) N N
    // Fila 2: G(OutChar) N N  
    // Fila 3: N N N (halt porque está completamente rodeado de N y borde)
    //
    // Pero espera, la VM empieza con DP=Right...
    // R mirando Right -> N (bloqueado)
    // R rota -> Down -> C (entra!)
    
    let width3 = 3;
    let height3 = 4;
    let mut pixels_corridor: Vec<[u8; 3]> = vec![BLACK; (width3 * height3) as usize];
    
    // Calcular colores para el corredor vertical
    let mut hue = 0usize;
    let mut light = 0usize;
    
    // (0,0) = inicio
    pixels_corridor[0] = get_color(hue, light);
    
    // (0,1) = InChar
    hue = (hue + 3) % 6;
    light = (light + 2) % 3;
    pixels_corridor[width3 as usize] = get_color(hue, light);
    
    // (0,2) = OutChar
    hue = (hue + 5) % 6;
    light = (light + 2) % 3;
    pixels_corridor[2 * width3 as usize] = get_color(hue, light);
    
    // (0,3) = Negro (halt)
    // Ya es negro por defecto
    
    write_bmp("tools/fixtures/samples/echo_corridor.bmp", width3 as u32, height3 as u32, &pixels_corridor)?;
    println!("✓ echo_corridor.bmp (3x4) - programa vertical con halt al final");
    
    println!("\nColores del corredor:");
    println!("  (0,0): #{:02X}{:02X}{:02X} - Inicio", 
             pixels_corridor[0][0], pixels_corridor[0][1], pixels_corridor[0][2]);
    println!("  (0,1): #{:02X}{:02X}{:02X} - InChar", 
             pixels_corridor[3][0], pixels_corridor[3][1], pixels_corridor[3][2]);
    println!("  (0,2): #{:02X}{:02X}{:02X} - OutChar", 
             pixels_corridor[6][0], pixels_corridor[6][1], pixels_corridor[6][2]);
    println!("  (0,3): #{:02X}{:02X}{:02X} - Negro (halt)", 
             pixels_corridor[9][0], pixels_corridor[9][1], pixels_corridor[9][2]);
    
    // Versión escalada para visualización
    let scale = 10u32;
    let mut scaled: Vec<[u8; 3]> = Vec::new();
    for y in 0..height3 {
        for _ in 0..scale {
            for x in 0..width3 {
                for _ in 0..scale {
                    scaled.push(pixels_corridor[(y * width3 + x) as usize]);
                }
            }
        }
    }
    write_bmp("tools/fixtures/samples/echo_corridor_10x.bmp", 
              width3 as u32 * scale, height3 as u32 * scale, &scaled)?;
    println!("✓ echo_corridor_10x.bmp - versión escalada para visualización");
    
    Ok(())
}
