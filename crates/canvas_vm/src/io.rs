use crate::error::VmError;

/// Trait for input sources - allows pluggable input implementations
pub trait InputSource {
    /// Read a number from input
    fn read_number(&mut self) -> Result<i32, VmError>;

    /// Read a character from input (as i32)
    fn read_char(&mut self) -> Result<i32, VmError>;

    /// Check if input is available
    fn has_input(&self) -> bool;

    /// Clear all inputs
    fn clear(&mut self);

    /// Rewind to beginning of input
    fn rewind(&mut self);

    /// Get remaining input count
    fn remaining(&self) -> usize;
}

/// Trait for output sinks - allows pluggable output implementations
pub trait OutputSink {
    /// Write a number to output
    fn write_number(&mut self, value: i32);

    /// Write a character to output
    fn write_char(&mut self, value: i32);

    /// Read all output as i32 values
    fn read(&self) -> Vec<i32>;

    /// Read all output as a string
    fn read_string(&self) -> String;

    /// Clear all output
    fn clear(&mut self);
}

/// Tipo de entrada para distinguir números de caracteres
#[derive(Debug, Clone, Copy)]
pub enum InputValue {
    Number(i32),
    Char(i32),
}

/// Tipo de salida para distinguir números de caracteres
#[derive(Debug, Clone, Copy)]
pub enum OutputValue {
    Number(i32),
    Char(i32),
}

/// Buffered input implementation - stores values in memory
#[derive(Debug, Clone)]
pub struct BufferedInput {
    buffer: Vec<InputValue>,
    position: usize,
}

impl BufferedInput {
    pub fn new() -> Self {
        Self {
            buffer: Vec::new(),
            position: 0,
        }
    }

    /// Create from text string (each char becomes an input)
    pub fn from_text(text: &str) -> Self {
        let mut input = Self::new();
        input.load_text(text);
        input
    }

    /// Create from numbers
    pub fn from_numbers(numbers: &[i32]) -> Self {
        let mut input = Self::new();
        input.load_number_vec(numbers);
        input
    }

    /// Write a single number value
    pub fn write(&mut self, value: i32) {
        self.buffer.push(InputValue::Number(value));
    }

    /// Write a single char value
    pub fn write_char(&mut self, c: char) {
        self.buffer.push(InputValue::Char(c as i32));
    }

    /// Load a string as character inputs (each char becomes an input)
    pub fn load_text(&mut self, text: &str) {
        for c in text.chars() {
            self.buffer.push(InputValue::Char(c as i32));
        }
    }

    /// Load a string as number inputs (parse whitespace-separated numbers)
    pub fn load_numbers(&mut self, text: &str) {
        for part in text.split_whitespace() {
            if let Ok(n) = part.parse::<i32>() {
                self.buffer.push(InputValue::Number(n));
            }
        }
    }

    /// Load a vector of numbers
    pub fn load_number_vec(&mut self, numbers: &[i32]) {
        for n in numbers {
            self.buffer.push(InputValue::Number(*n));
        }
    }

    pub fn read(&mut self) -> Option<i32> {
        if self.position < self.buffer.len() {
            let value = match self.buffer[self.position] {
                InputValue::Number(n) => n,
                InputValue::Char(c) => c,
            };
            self.position += 1;
            Some(value)
        } else {
            None
        }
    }
}

impl InputSource for BufferedInput {
    fn read_number(&mut self) -> Result<i32, VmError> {
        self.read().ok_or(VmError::InvalidInput)
    }

    fn read_char(&mut self) -> Result<i32, VmError> {
        self.read().ok_or(VmError::InvalidInput)
    }

    fn has_input(&self) -> bool {
        self.position < self.buffer.len()
    }

    fn clear(&mut self) {
        self.buffer.clear();
        self.position = 0;
    }

    fn rewind(&mut self) {
        self.position = 0;
    }

    fn remaining(&self) -> usize {
        self.buffer.len().saturating_sub(self.position)
    }
}

// Type alias for backward compatibility
pub type Input = BufferedInput;

/// Buffered output implementation - stores values in memory
#[derive(Debug, Clone)]
pub struct BufferedOutput {
    buffer: Vec<OutputValue>,
}

impl BufferedOutput {
    pub fn new() -> Self {
        Self { buffer: Vec::new() }
    }
}

impl OutputSink for BufferedOutput {
    fn write_number(&mut self, value: i32) {
        self.buffer.push(OutputValue::Number(value));
    }

    fn write_char(&mut self, value: i32) {
        self.buffer.push(OutputValue::Char(value));
    }

    fn read(&self) -> Vec<i32> {
        self.buffer
            .iter()
            .map(|v| match v {
                OutputValue::Number(n) => *n,
                OutputValue::Char(c) => *c,
            })
            .collect()
    }

    fn read_string(&self) -> String {
        self.buffer
            .iter()
            .map(|v| match v {
                OutputValue::Number(n) => n.to_string(),
                OutputValue::Char(c) => char::from_u32(*c as u32)
                    .map(|ch| ch.to_string())
                    .unwrap_or_default(),
            })
            .collect()
    }

    fn clear(&mut self) {
        self.buffer.clear();
    }
}

// Type alias for backward compatibility
pub type Output = BufferedOutput;

impl Default for BufferedInput {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for BufferedOutput {
    fn default() -> Self {
        Self::new()
    }
}
