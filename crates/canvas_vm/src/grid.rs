use crate::error::VmError;
use crate::exits::{CodelChooser, Direction, Position};
use crate::ops::PietColor;
use std::collections::{HashMap, HashSet};

/// Type to identify blocks
pub type BlockId = usize;

/// Precomputed block information
#[derive(Debug, Clone)]
pub struct BlockInfo {
    pub size: usize,
    pub color: PietColor,
    pub positions: HashSet<Position>,
}

/// Key for precomputed exits
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ExitKey {
    block_id: BlockId,
    dp: Direction,
    cc: CodelChooser,
}

/// Piet color grid with precomputed blocks and exits
#[derive(Debug, Clone)]
pub struct Grid {
    width: usize,
    height: usize,
    cells: Vec<PietColor>,
    // Precomputation
    block_ids: Vec<BlockId>,              // blockId[x][y]
    blocks: HashMap<BlockId, BlockInfo>,   // blockSize[block]
    exits: HashMap<ExitKey, Option<Position>>, // exit[block][dp][cc]
}

impl Grid {
    /// Creates a grid from a vector of colors
    pub fn new(width: usize, height: usize, cells: Vec<PietColor>) -> Result<Self, VmError> {
        if cells.len() != width * height {
            return Err(VmError::OutOfBounds);
        }
        
        let mut grid = Self {
            width,
            height,
            cells,
            block_ids: vec![0; width * height],
            blocks: HashMap::new(),
            exits: HashMap::new(),
        };
        
        grid.precompute_blocks();
        grid.precompute_exits();
        
        Ok(grid)
    }

    /// Creates a grid from RGBA data (each pixel = 4 bytes)
    pub fn from_rgba(width: usize, height: usize, rgba_data: &[u8]) -> Result<Self, VmError> {
        Self::from_rgba_with_codel_size(width, height, rgba_data, None)
    }
    
    /// Detects the codel size from RGBA data without creating the grid
    /// Returns 1 if detection is uncertain
    pub fn detect_codel_size_from_rgba(width: usize, height: usize, rgba_data: &[u8]) -> usize {
        if rgba_data.len() != width * height * 4 {
            return 1;
        }
        
        let get_pixel = |x: usize, y: usize| -> (u8, u8, u8) {
            let idx = (y * width + x) * 4;
            (rgba_data[idx], rgba_data[idx + 1], rgba_data[idx + 2])
        };
        
        Self::detect_codel_size(width, height, &get_pixel)
    }
    
    /// Creates a grid from RGBA data with optional codel size
    /// If codel_size is None, it will be auto-detected
    pub fn from_rgba_with_codel_size(
        width: usize, 
        height: usize, 
        rgba_data: &[u8],
        codel_size: Option<usize>
    ) -> Result<Self, VmError> {
        if rgba_data.len() != width * height * 4 {
            return Err(VmError::OutOfBounds);
        }

        // Helper to get pixel color at (x, y)
        let get_pixel = |x: usize, y: usize| -> (u8, u8, u8) {
            let idx = (y * width + x) * 4;
            (rgba_data[idx], rgba_data[idx + 1], rgba_data[idx + 2])
        };

        // Detect or use provided codel size
        let cs = codel_size.unwrap_or_else(|| Self::detect_codel_size(width, height, &get_pixel));
        
        if cs == 1 {
            // No reduction needed
            let mut cells = Vec::with_capacity(width * height);
            for i in 0..(width * height) {
                let idx = i * 4;
                let r = rgba_data[idx];
                let g = rgba_data[idx + 1];
                let b = rgba_data[idx + 2];
                cells.push(PietColor::from_rgb(r, g, b)?);
            }
            return Self::new(width, height, cells);
        }
        
        // Reduce the grid by codel size
        let new_width = width / cs;
        let new_height = height / cs;
        
        if new_width == 0 || new_height == 0 {
            return Err(VmError::OutOfBounds);
        }
        
        let mut cells = Vec::with_capacity(new_width * new_height);
        for cy in 0..new_height {
            for cx in 0..new_width {
                // Sample the top-left pixel of each codel
                let (r, g, b) = get_pixel(cx * cs, cy * cs);
                cells.push(PietColor::from_rgb(r, g, b)?);
            }
        }
        
        Self::new(new_width, new_height, cells)
    }
    
    /// Detects the codel size by finding the GCD of color run lengths
    /// Uses multiple scan lines for more accurate detection
    fn detect_codel_size<F>(width: usize, height: usize, get_pixel: &F) -> usize 
    where F: Fn(usize, usize) -> (u8, u8, u8)
    {
        let mut run_lengths = Vec::new();
        
        // Scan multiple rows for horizontal runs (first, middle, last)
        let rows_to_scan = [0, height / 2, height.saturating_sub(1)];
        for &row in &rows_to_scan {
            if row >= height {
                continue;
            }
            let mut x = 0;
            while x < width {
                let color = get_pixel(x, row);
                let mut run_len = 1;
                while x + run_len < width && get_pixel(x + run_len, row) == color {
                    run_len += 1;
                }
                run_lengths.push(run_len);
                x += run_len;
            }
        }
        
        // Scan multiple columns for vertical runs (first, middle, last)
        let cols_to_scan = [0, width / 2, width.saturating_sub(1)];
        for &col in &cols_to_scan {
            if col >= width {
                continue;
            }
            let mut y = 0;
            while y < height {
                let color = get_pixel(col, y);
                let mut run_len = 1;
                while y + run_len < height && get_pixel(col, y + run_len) == color {
                    run_len += 1;
                }
                run_lengths.push(run_len);
                y += run_len;
            }
        }
        
        // Also check for uniform color blocks by sampling corners
        // If we detect that all corners of potential codels have the same color, 
        // that confirms the codel size
        let candidate_sizes = Self::find_candidate_codel_sizes(width, height, get_pixel);
        for size in candidate_sizes {
            run_lengths.push(size);
        }
        
        // Find GCD of all run lengths
        if run_lengths.is_empty() {
            return 1;
        }
        
        let mut result = run_lengths[0];
        for &len in &run_lengths[1..] {
            result = Self::gcd(result, len);
            if result == 1 {
                break;
            }
        }
        
        result.max(1)
    }
    
    /// Find candidate codel sizes by checking if the image can be evenly divided
    /// into uniform color blocks of a given size
    fn find_candidate_codel_sizes<F>(width: usize, height: usize, get_pixel: &F) -> Vec<usize>
    where F: Fn(usize, usize) -> (u8, u8, u8)
    {
        let mut candidates = Vec::new();
        
        // Common codel sizes to check
        let sizes_to_check = [2, 4, 5, 8, 10, 16, 20, 25, 32];
        
        for &size in &sizes_to_check {
            if width % size != 0 || height % size != 0 {
                continue;
            }
            
            // Check if all blocks at this size are uniform color
            let mut is_valid = true;
            'outer: for cy in 0..(height / size) {
                for cx in 0..(width / size) {
                    let base_x = cx * size;
                    let base_y = cy * size;
                    let base_color = get_pixel(base_x, base_y);
                    
                    // Check all pixels in this block
                    for dy in 0..size {
                        for dx in 0..size {
                            if get_pixel(base_x + dx, base_y + dy) != base_color {
                                is_valid = false;
                                break 'outer;
                            }
                        }
                    }
                }
            }
            
            if is_valid {
                candidates.push(size);
            }
        }
        
        candidates
    }
    
    /// Greatest common divisor
    fn gcd(a: usize, b: usize) -> usize {
        if b == 0 { a } else { Self::gcd(b, a % b) }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn get(&self, pos: Position) -> Option<PietColor> {
        if pos.x < self.width && pos.y < self.height {
            Some(self.cells[pos.y * self.width + pos.x])
        } else {
            None
        }
    }
    
    /// Gets the block ID at a position
    pub fn get_block_id(&self, pos: Position) -> Option<BlockId> {
        if pos.x < self.width && pos.y < self.height {
            Some(self.block_ids[pos.y * self.width + pos.x])
        } else {
            None
        }
    }
    
    /// Gets block information by its ID
    pub fn get_block_info(&self, block_id: BlockId) -> Option<&BlockInfo> {
        self.blocks.get(&block_id)
    }
    
    /// Gets the precomputed exit for a block
    pub fn get_exit(&self, block_id: BlockId, dp: Direction, cc: CodelChooser) -> Option<Position> {
        let key = ExitKey { block_id, dp, cc };
        self.exits.get(&key).copied().flatten()
    }

    /// Precomputes all blocks using flood-fill
    fn precompute_blocks(&mut self) {
        let mut visited = vec![false; self.width * self.height];
        let mut next_block_id: BlockId = 0;
        
        for y in 0..self.height {
            for x in 0..self.width {
                let idx = y * self.width + x;
                if visited[idx] {
                    continue;
                }
                
                let pos = Position::new(x, y);
                let color = self.cells[idx];
                
                // Flood-fill to find the block
                let positions = self.flood_fill(pos, color, &mut visited);
                let size = positions.len();
                
                // Assign block_id to all positions
                for &p in &positions {
                    self.block_ids[p.y * self.width + p.x] = next_block_id;
                }
                
                // Save block information
                self.blocks.insert(next_block_id, BlockInfo {
                    size,
                    color,
                    positions,
                });
                
                next_block_id += 1;
            }
        }
    }
    
    /// Flood-fill to find contiguous blocks
    fn flood_fill(&self, start: Position, color: PietColor, visited: &mut [bool]) -> HashSet<Position> {
        let mut block = HashSet::new();
        let mut to_visit = vec![start];
        
        while let Some(pos) = to_visit.pop() {
            let idx = pos.y * self.width + pos.x;
            if visited[idx] {
                continue;
            }
            
            if self.get(pos) == Some(color) {
                visited[idx] = true;
                block.insert(pos);
                
                // Add neighbors (4-connectivity)
                for dir in [Direction::Right, Direction::Down, Direction::Left, Direction::Up] {
                    if let Some(next) = pos.step(dir, self.width, self.height) {
                        let next_idx = next.y * self.width + next.x;
                        if !visited[next_idx] {
                            to_visit.push(next);
                        }
                    }
                }
            }
        }
        
        block
    }
    
    /// Precomputes all possible exits
    fn precompute_exits(&mut self) {
        for (&block_id, block_info) in &self.blocks {
            for dp in [Direction::Right, Direction::Down, Direction::Left, Direction::Up] {
                for cc in [CodelChooser::Left, CodelChooser::Right] {
                    let exit = self.find_exit_for_block(block_info, dp, cc);
                    let key = ExitKey { block_id, dp, cc };
                    self.exits.insert(key, exit);
                }
            }
        }
    }
    
    /// Finds the exit codel of a block (used internally for precomputation)
    fn find_exit_for_block(
        &self,
        block_info: &BlockInfo,
        dp: Direction,
        cc: CodelChooser,
    ) -> Option<Position> {
        if block_info.positions.is_empty() {
            return None;
        }

        // Determinar el eje de búsqueda según DP
        let positions: Vec<Position> = match dp {
            Direction::Right => {
                let max_x = block_info.positions.iter().map(|p| p.x).max()?;
                let mut candidates: Vec<_> = block_info.positions.iter()
                    .filter(|p| p.x == max_x)
                    .copied()
                    .collect();
                if cc == CodelChooser::Left {
                    candidates.sort_by_key(|p| p.y);
                } else {
                    candidates.sort_by_key(|p| std::cmp::Reverse(p.y));
                }
                candidates
            }
            Direction::Down => {
                let max_y = block_info.positions.iter().map(|p| p.y).max()?;
                let mut candidates: Vec<_> = block_info.positions.iter()
                    .filter(|p| p.y == max_y)
                    .copied()
                    .collect();
                if cc == CodelChooser::Left {
                    candidates.sort_by_key(|p| std::cmp::Reverse(p.x));
                } else {
                    candidates.sort_by_key(|p| p.x);
                }
                candidates
            }
            Direction::Left => {
                let min_x = block_info.positions.iter().map(|p| p.x).min()?;
                let mut candidates: Vec<_> = block_info.positions.iter()
                    .filter(|p| p.x == min_x)
                    .copied()
                    .collect();
                if cc == CodelChooser::Left {
                    candidates.sort_by_key(|p| std::cmp::Reverse(p.y));
                } else {
                    candidates.sort_by_key(|p| p.y);
                }
                candidates
            }
            Direction::Up => {
                let min_y = block_info.positions.iter().map(|p| p.y).min()?;
                let mut candidates: Vec<_> = block_info.positions.iter()
                    .filter(|p| p.y == min_y)
                    .copied()
                    .collect();
                if cc == CodelChooser::Left {
                    candidates.sort_by_key(|p| p.x);
                } else {
                    candidates.sort_by_key(|p| std::cmp::Reverse(p.x));
                }
                candidates
            }
        };

        let exit = positions.first().copied()?;
        
        // Retornar la posición siguiente en dirección DP (incluso si es negro)
        // La VM decidirá qué hacer con el color
        exit.step(dp, self.width, self.height)
    }

    /// Encuentra todos los codels contiguos del mismo color (legacy, para tests)
    #[allow(dead_code)]
    pub fn find_block(&self, start: Position) -> HashSet<Position> {
        let color = match self.get(start) {
            Some(c) => c,
            None => return HashSet::new(),
        };

        let mut block = HashSet::new();
        let mut to_visit = vec![start];
        let mut visited = HashSet::new();

        while let Some(pos) = to_visit.pop() {
            if visited.contains(&pos) {
                continue;
            }
            visited.insert(pos);

            if self.get(pos) == Some(color) {
                block.insert(pos);

                // Revisar vecinos (4-conectividad)
                for dir in [Direction::Right, Direction::Down, Direction::Left, Direction::Up] {
                    if let Some(next) = pos.step(dir, self.width, self.height) {
                        if !visited.contains(&next) {
                            to_visit.push(next);
                        }
                    }
                }
            }
        }

        block
    }

    /// Encuentra el codel de salida del bloque según DP y CC (legacy, para tests)
    #[allow(dead_code)]
    pub fn find_exit(
        &self,
        block: &HashSet<Position>,
        dp: Direction,
        cc: CodelChooser,
    ) -> Option<Position> {
        if block.is_empty() {
            return None;
        }

        // Determinar el eje de búsqueda según DP
        let positions: Vec<Position> = match dp {
            Direction::Right => {
                // Buscar el x máximo
                let max_x = block.iter().map(|p| p.x).max()?;
                let mut candidates: Vec<_> = block.iter().filter(|p| p.x == max_x).copied().collect();
                // Ordenar por y según CC
                if cc == CodelChooser::Left {
                    candidates.sort_by_key(|p| p.y); // Arriba primero
                } else {
                    candidates.sort_by_key(|p| std::cmp::Reverse(p.y)); // Abajo primero
                }
                candidates
            }
            Direction::Down => {
                let max_y = block.iter().map(|p| p.y).max()?;
                let mut candidates: Vec<_> = block.iter().filter(|p| p.y == max_y).copied().collect();
                if cc == CodelChooser::Left {
                    candidates.sort_by_key(|p| std::cmp::Reverse(p.x)); // Derecha primero
                } else {
                    candidates.sort_by_key(|p| p.x); // Izquierda primero
                }
                candidates
            }
            Direction::Left => {
                let min_x = block.iter().map(|p| p.x).min()?;
                let mut candidates: Vec<_> = block.iter().filter(|p| p.x == min_x).copied().collect();
                if cc == CodelChooser::Left {
                    candidates.sort_by_key(|p| std::cmp::Reverse(p.y)); // Abajo primero
                } else {
                    candidates.sort_by_key(|p| p.y); // Arriba primero
                }
                candidates
            }
            Direction::Up => {
                let min_y = block.iter().map(|p| p.y).min()?;
                let mut candidates: Vec<_> = block.iter().filter(|p| p.y == min_y).copied().collect();
                if cc == CodelChooser::Left {
                    candidates.sort_by_key(|p| p.x); // Izquierda primero
                } else {
                    candidates.sort_by_key(|p| std::cmp::Reverse(p.x)); // Derecha primero
                }
                candidates
            }
        };

        positions.first().copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grid_creation() {
        let cells = vec![PietColor::Red; 9];
        let grid = Grid::new(3, 3, cells).unwrap();
        assert_eq!(grid.width(), 3);
        assert_eq!(grid.height(), 3);
    }

    #[test]
    fn test_grid_get() {
        let cells = vec![
            PietColor::Red, PietColor::Blue, PietColor::Green,
            PietColor::Yellow, PietColor::White, PietColor::Black,
            PietColor::Red, PietColor::Red, PietColor::Red,
        ];
        let grid = Grid::new(3, 3, cells).unwrap();
        
        assert_eq!(grid.get(Position::new(0, 0)), Some(PietColor::Red));
        assert_eq!(grid.get(Position::new(1, 0)), Some(PietColor::Blue));
        assert_eq!(grid.get(Position::new(1, 1)), Some(PietColor::White));
        assert_eq!(grid.get(Position::new(3, 3)), None);
    }

    #[test]
    fn test_find_block() {
        let cells = vec![
            PietColor::Red, PietColor::Red, PietColor::Blue,
            PietColor::Red, PietColor::Blue, PietColor::Blue,
            PietColor::Green, PietColor::Blue, PietColor::Blue,
        ];
        let grid = Grid::new(3, 3, cells).unwrap();
        
        let block = grid.find_block(Position::new(0, 0));
        assert_eq!(block.len(), 3); // 3 rojos conectados
        assert!(block.contains(&Position::new(0, 0)));
        assert!(block.contains(&Position::new(1, 0)));
        assert!(block.contains(&Position::new(0, 1)));
    }

    #[test]
    fn test_find_exit() {
        let cells = vec![
            PietColor::Red, PietColor::Red, PietColor::Blue,
            PietColor::Red, PietColor::Blue, PietColor::Blue,
            PietColor::Green, PietColor::Blue, PietColor::Blue,
        ];
        let grid = Grid::new(3, 3, cells).unwrap();
        
        let block = grid.find_block(Position::new(0, 0));
        
        // DP=Right, CC=Left debería dar (1, 0) - el más a la derecha y arriba
        let exit = grid.find_exit(&block, Direction::Right, CodelChooser::Left);
        assert_eq!(exit, Some(Position::new(1, 0)));
    }

    #[test]
    fn test_detect_codel_size_1px() {
        // 3x3 imagen con codel size 1 (cada pixel es un codel)
        let rgba = vec![
            255, 0, 0, 255,  0, 255, 0, 255,  0, 0, 255, 255,
            255, 255, 0, 255,  255, 255, 255, 255,  0, 0, 0, 255,
            0, 0, 0, 255,  255, 0, 0, 255,  0, 255, 0, 255,
        ];
        let cs = Grid::detect_codel_size_from_rgba(3, 3, &rgba);
        assert_eq!(cs, 1);
    }

    #[test]
    fn test_detect_codel_size_2px() {
        // 4x4 imagen donde cada codel es 2x2 pixels
        // Codel grid: 2x2
        // [Red,  Blue ]
        // [Green, Yellow]
        let rgba = vec![
            // Row 0 (2 pixels height)
            255, 0, 0, 255,  255, 0, 0, 255,  0, 0, 255, 255,  0, 0, 255, 255,  // Red Red Blue Blue
            255, 0, 0, 255,  255, 0, 0, 255,  0, 0, 255, 255,  0, 0, 255, 255,  // Red Red Blue Blue
            // Row 1 (2 pixels height)
            0, 255, 0, 255,  0, 255, 0, 255,  255, 255, 0, 255,  255, 255, 0, 255,  // Green Green Yellow Yellow
            0, 255, 0, 255,  0, 255, 0, 255,  255, 255, 0, 255,  255, 255, 0, 255,  // Green Green Yellow Yellow
        ];
        let cs = Grid::detect_codel_size_from_rgba(4, 4, &rgba);
        assert_eq!(cs, 2);
    }

    #[test]
    fn test_grid_from_rgba_with_codel_size() {
        // 4x4 imagen donde cada codel es 2x2 pixels
        let rgba = vec![
            // Red Red Blue Blue  (2x2 blocks)
            255, 0, 0, 255,  255, 0, 0, 255,  0, 0, 255, 255,  0, 0, 255, 255,
            255, 0, 0, 255,  255, 0, 0, 255,  0, 0, 255, 255,  0, 0, 255, 255,
            // Green Green Yellow Yellow (2x2 blocks)
            0, 255, 0, 255,  0, 255, 0, 255,  255, 255, 0, 255,  255, 255, 0, 255,
            0, 255, 0, 255,  0, 255, 0, 255,  255, 255, 0, 255,  255, 255, 0, 255,
        ];
        
        // Con codel size 2, debería reducir a 2x2
        let grid = Grid::from_rgba_with_codel_size(4, 4, &rgba, Some(2)).unwrap();
        assert_eq!(grid.width(), 2);
        assert_eq!(grid.height(), 2);
        assert_eq!(grid.get(Position::new(0, 0)), Some(PietColor::Red));
        assert_eq!(grid.get(Position::new(1, 0)), Some(PietColor::Blue));
        assert_eq!(grid.get(Position::new(0, 1)), Some(PietColor::Green));
        assert_eq!(grid.get(Position::new(1, 1)), Some(PietColor::Yellow));
    }

    #[test]
    fn test_grid_from_rgba_auto_detect() {
        // 4x4 imagen donde cada codel es 2x2 pixels
        let rgba = vec![
            255, 0, 0, 255,  255, 0, 0, 255,  0, 0, 255, 255,  0, 0, 255, 255,
            255, 0, 0, 255,  255, 0, 0, 255,  0, 0, 255, 255,  0, 0, 255, 255,
            0, 255, 0, 255,  0, 255, 0, 255,  255, 255, 0, 255,  255, 255, 0, 255,
            0, 255, 0, 255,  0, 255, 0, 255,  255, 255, 0, 255,  255, 255, 0, 255,
        ];
        
        // Auto-detect debería encontrar codel size 2
        let grid = Grid::from_rgba_with_codel_size(4, 4, &rgba, None).unwrap();
        assert_eq!(grid.width(), 2);
        assert_eq!(grid.height(), 2);
    }
}
