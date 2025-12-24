use crate::error::VmError;

/// The 20 colors of the Piet language: 18 chromatic + white + black
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PietColor {
    // Hue 0: Red
    LightRed,
    Red,
    DarkRed,
    // Hue 1: Yellow
    LightYellow,
    Yellow,
    DarkYellow,
    // Hue 2: Green
    LightGreen,
    Green,
    DarkGreen,
    // Hue 3: Cyan
    LightCyan,
    Cyan,
    DarkCyan,
    // Hue 4: Blue
    LightBlue,
    Blue,
    DarkBlue,
    // Hue 5: Magenta
    LightMagenta,
    Magenta,
    DarkMagenta,
    // Special colors
    White,
    Black,
}

impl PietColor {
    /// Converts RGB to PietColor
    pub fn from_rgb(r: u8, g: u8, b: u8) -> Result<Self, VmError> {
        match (r, g, b) {
            // Light Red
            (0xFF, 0xC0, 0xC0) => Ok(PietColor::LightRed),
            // Red
            (0xFF, 0x00, 0x00) => Ok(PietColor::Red),
            // Dark Red
            (0xC0, 0x00, 0x00) => Ok(PietColor::DarkRed),
            
            // Light Yellow
            (0xFF, 0xFF, 0xC0) => Ok(PietColor::LightYellow),
            // Yellow
            (0xFF, 0xFF, 0x00) => Ok(PietColor::Yellow),
            // Dark Yellow
            (0xC0, 0xC0, 0x00) => Ok(PietColor::DarkYellow),
            
            // Light Green
            (0xC0, 0xFF, 0xC0) => Ok(PietColor::LightGreen),
            // Green
            (0x00, 0xFF, 0x00) => Ok(PietColor::Green),
            // Dark Green
            (0x00, 0xC0, 0x00) => Ok(PietColor::DarkGreen),
            
            // Light Cyan
            (0xC0, 0xFF, 0xFF) => Ok(PietColor::LightCyan),
            // Cyan
            (0x00, 0xFF, 0xFF) => Ok(PietColor::Cyan),
            // Dark Cyan
            (0x00, 0xC0, 0xC0) => Ok(PietColor::DarkCyan),
            
            // Light Blue
            (0xC0, 0xC0, 0xFF) => Ok(PietColor::LightBlue),
            // Blue
            (0x00, 0x00, 0xFF) => Ok(PietColor::Blue),
            // Dark Blue
            (0x00, 0x00, 0xC0) => Ok(PietColor::DarkBlue),
            
            // Light Magenta
            (0xFF, 0xC0, 0xFF) => Ok(PietColor::LightMagenta),
            // Magenta
            (0xFF, 0x00, 0xFF) => Ok(PietColor::Magenta),
            // Dark Magenta
            (0xC0, 0x00, 0xC0) => Ok(PietColor::DarkMagenta),
            
            // White
            (0xFF, 0xFF, 0xFF) => Ok(PietColor::White),
            // Black
            (0x00, 0x00, 0x00) => Ok(PietColor::Black),
            
            // Any other color is treated as black (blocked)
            // This allows handling non-standard colors in Piet images
            _ => Ok(PietColor::Black),
        }
    }

    /// Gets the hue of the color (0-5), None for white/black
    pub fn hue(&self) -> Option<u8> {
        match self {
            PietColor::LightRed | PietColor::Red | PietColor::DarkRed => Some(0),
            PietColor::LightYellow | PietColor::Yellow | PietColor::DarkYellow => Some(1),
            PietColor::LightGreen | PietColor::Green | PietColor::DarkGreen => Some(2),
            PietColor::LightCyan | PietColor::Cyan | PietColor::DarkCyan => Some(3),
            PietColor::LightBlue | PietColor::Blue | PietColor::DarkBlue => Some(4),
            PietColor::LightMagenta | PietColor::Magenta | PietColor::DarkMagenta => Some(5),
            PietColor::White | PietColor::Black => None,
        }
    }

    /// Gets the lightness of the color (0=light, 1=normal, 2=dark), None for white/black
    pub fn lightness(&self) -> Option<u8> {
        match self {
            PietColor::LightRed | PietColor::LightYellow | PietColor::LightGreen |
            PietColor::LightCyan | PietColor::LightBlue | PietColor::LightMagenta => Some(0),
            
            PietColor::Red | PietColor::Yellow | PietColor::Green |
            PietColor::Cyan | PietColor::Blue | PietColor::Magenta => Some(1),
            
            PietColor::DarkRed | PietColor::DarkYellow | PietColor::DarkGreen |
            PietColor::DarkCyan | PietColor::DarkBlue | PietColor::DarkMagenta => Some(2),
            
            PietColor::White | PietColor::Black => None,
        }
    }

    pub fn is_white(&self) -> bool {
        matches!(self, PietColor::White)
    }

    pub fn is_black(&self) -> bool {
        matches!(self, PietColor::Black)
    }
}

/// Calculates the operation resulting from hue and lightness change
/// Based on the official Piet specification:
/// https://www.dangermouse.net/esoteric/piet.html
///
/// | Light\Hue | 0      | 1        | 2        | 3        | 4          | 5          |
/// |-----------|--------|----------|----------|----------|------------|------------|
/// | 0         | -      | push     | pop      | add      | subtract   | multiply   |
/// | 1         | divide | mod      | not      | greater  | pointer    | switch     |
/// | 2         | dup    | roll     | in(num)  | in(char) | out(num)   | out(char)  |
pub fn get_operation(hue_change: i8, lightness_change: i8) -> Option<Operation> {
    // Normalize changes to [0, 5] for hue and [0, 2] for lightness
    let hue = ((hue_change % 6 + 6) % 6) as u8;
    let light = ((lightness_change % 3 + 3) % 3) as u8;
    
    match (hue, light) {
        // Lightness 0
        (0, 0) => None,                      // No change = no operation
        (1, 0) => Some(Operation::Push),
        (2, 0) => Some(Operation::Pop),
        (3, 0) => Some(Operation::Add),
        (4, 0) => Some(Operation::Subtract),
        (5, 0) => Some(Operation::Multiply),
        // Lightness 1
        (0, 1) => Some(Operation::Divide),
        (1, 1) => Some(Operation::Mod),
        (2, 1) => Some(Operation::Not),
        (3, 1) => Some(Operation::Greater),
        (4, 1) => Some(Operation::Pointer),
        (5, 1) => Some(Operation::Switch),
        // Lightness 2
        (0, 2) => Some(Operation::Duplicate),
        (1, 2) => Some(Operation::Roll),
        (2, 2) => Some(Operation::InNumber),
        (3, 2) => Some(Operation::InChar),
        (4, 2) => Some(Operation::OutNumber),
        (5, 2) => Some(Operation::OutChar),
        _ => None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operation {
    Push,
    Pop,
    Add,
    Subtract,
    Multiply,
    Divide,
    Mod,
    Not,
    Greater,
    Pointer,
    Switch,
    Duplicate,
    Roll,
    InNumber,
    InChar,
    OutNumber,
    OutChar,
}

impl Operation {
    #[allow(dead_code)]
    pub fn name(&self) -> &'static str {
        match self {
            Operation::Push => "push",
            Operation::Pop => "pop",
            Operation::Add => "add",
            Operation::Subtract => "subtract",
            Operation::Multiply => "multiply",
            Operation::Divide => "divide",
            Operation::Mod => "mod",
            Operation::Not => "not",
            Operation::Greater => "greater",
            Operation::Pointer => "pointer",
            Operation::Switch => "switch",
            Operation::Duplicate => "duplicate",
            Operation::Roll => "roll",
            Operation::InNumber => "in(number)",
            Operation::InChar => "in(char)",
            Operation::OutNumber => "out(number)",
            Operation::OutChar => "out(char)",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_from_rgb() {
        assert_eq!(PietColor::from_rgb(0xFF, 0x00, 0x00).unwrap(), PietColor::Red);
        assert_eq!(PietColor::from_rgb(0xFF, 0xFF, 0x00).unwrap(), PietColor::Yellow);
        assert_eq!(PietColor::from_rgb(0x00, 0x00, 0xFF).unwrap(), PietColor::Blue);
        assert_eq!(PietColor::from_rgb(0xFF, 0xFF, 0xFF).unwrap(), PietColor::White);
        assert_eq!(PietColor::from_rgb(0x00, 0x00, 0x00).unwrap(), PietColor::Black);
    }

    #[test]
    fn test_hue_and_lightness() {
        assert_eq!(PietColor::Red.hue(), Some(0));
        assert_eq!(PietColor::LightRed.lightness(), Some(0));
        assert_eq!(PietColor::Red.lightness(), Some(1));
        assert_eq!(PietColor::DarkRed.lightness(), Some(2));
        assert_eq!(PietColor::White.hue(), None);
        assert_eq!(PietColor::Black.hue(), None);
    }

    #[test]
    fn test_operations() {
        assert_eq!(get_operation(0, 1), Some(Operation::Push));
        assert_eq!(get_operation(0, 2), Some(Operation::Pop));
        assert_eq!(get_operation(1, 0), Some(Operation::Add));
        assert_eq!(get_operation(1, 1), Some(Operation::Subtract));
        assert_eq!(get_operation(1, 2), Some(Operation::Multiply));
        assert_eq!(get_operation(2, 0), Some(Operation::Divide));
        assert_eq!(get_operation(4, 0), Some(Operation::Duplicate));
        assert_eq!(get_operation(5, 2), Some(Operation::OutChar));
    }
}
