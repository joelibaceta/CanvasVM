use serde::{Deserialize, Serialize};

/// Direction Pointer - indica la dirección de movimiento del puntero
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Direction {
    Right = 0,
    Down = 1,
    Left = 2,
    Up = 3,
}

impl Direction {
    /// Rota el DP en sentido horario n veces
    pub fn rotate_clockwise(self, n: i32) -> Self {
        let current = self as i32;
        let new = (current + n).rem_euclid(4);
        match new {
            0 => Direction::Right,
            1 => Direction::Down,
            2 => Direction::Left,
            3 => Direction::Up,
            _ => unreachable!(),
        }
    }

    /// Returns the delta (dx, dy) for this direction
    pub fn delta(self) -> (i32, i32) {
        match self {
            Direction::Right => (1, 0),
            Direction::Down => (0, 1),
            Direction::Left => (-1, 0),
            Direction::Up => (0, -1),
        }
    }
}

/// Codel Chooser - indica qué lado del bloque elegir para salir
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CodelChooser {
    Left = 0,
    Right = 1,
}

impl CodelChooser {
    /// Alterna el CC
    pub fn toggle(self) -> Self {
        match self {
            CodelChooser::Left => CodelChooser::Right,
            CodelChooser::Right => CodelChooser::Left,
        }
    }
}

/// Representa una posición en la grilla
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

impl Position {
    pub fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }

    /// Mueve la posición en la dirección dada, si es válido dentro de los límites
    pub fn step(&self, dir: Direction, width: usize, height: usize) -> Option<Position> {
        let (dx, dy) = dir.delta();
        let new_x = self.x as i32 + dx;
        let new_y = self.y as i32 + dy;

        if new_x >= 0 && new_y >= 0 && (new_x as usize) < width && (new_y as usize) < height {
            Some(Position::new(new_x as usize, new_y as usize))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_direction_rotation() {
        assert_eq!(Direction::Right.rotate_clockwise(1), Direction::Down);
        assert_eq!(Direction::Down.rotate_clockwise(1), Direction::Left);
        assert_eq!(Direction::Left.rotate_clockwise(1), Direction::Up);
        assert_eq!(Direction::Up.rotate_clockwise(1), Direction::Right);
        
        assert_eq!(Direction::Right.rotate_clockwise(4), Direction::Right);
        assert_eq!(Direction::Right.rotate_clockwise(-1), Direction::Up);
    }

    #[test]
    fn test_direction_delta() {
        assert_eq!(Direction::Right.delta(), (1, 0));
        assert_eq!(Direction::Down.delta(), (0, 1));
        assert_eq!(Direction::Left.delta(), (-1, 0));
        assert_eq!(Direction::Up.delta(), (0, -1));
    }

    #[test]
    fn test_codel_chooser_toggle() {
        assert_eq!(CodelChooser::Left.toggle(), CodelChooser::Right);
        assert_eq!(CodelChooser::Right.toggle(), CodelChooser::Left);
    }

    #[test]
    fn test_position_step() {
        let pos = Position::new(5, 5);
        assert_eq!(pos.step(Direction::Right, 10, 10), Some(Position::new(6, 5)));
        assert_eq!(pos.step(Direction::Down, 10, 10), Some(Position::new(5, 6)));
        assert_eq!(pos.step(Direction::Left, 10, 10), Some(Position::new(4, 5)));
        assert_eq!(pos.step(Direction::Up, 10, 10), Some(Position::new(5, 4)));
        
        // Fuera de límites
        let edge = Position::new(0, 0);
        assert_eq!(edge.step(Direction::Left, 10, 10), None);
        assert_eq!(edge.step(Direction::Up, 10, 10), None);
    }
}
