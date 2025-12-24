use std::fmt;

#[derive(Debug)]
pub enum VmError {
    InvalidColor(u8, u8, u8),
    DivisionByZero,
    EmptyStack,
    StackUnderflow,
    InvalidInput,
    OutOfBounds,
    Halted,
    /// Watchdog timeout - programa excedió el límite de pasos
    ExecutionTimeout(usize),
}

impl fmt::Display for VmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VmError::InvalidColor(r, g, b) => write!(f, "Invalid Piet color: RGB({}, {}, {})", r, g, b),
            VmError::DivisionByZero => write!(f, "Division by zero"),
            VmError::EmptyStack => write!(f, "Stack is empty"),
            VmError::StackUnderflow => write!(f, "Stack underflow"),
            VmError::InvalidInput => write!(f, "Invalid input"),
            VmError::OutOfBounds => write!(f, "Position out of bounds"),
            VmError::Halted => write!(f, "VM is halted"),
            VmError::ExecutionTimeout(steps) => write!(f, "Execution timeout after {} steps", steps),
        }
    }
}

impl std::error::Error for VmError {}
