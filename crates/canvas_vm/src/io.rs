use crate::error::VmError;

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

/// Sistema de entrada/salida para la VM
#[derive(Debug, Clone)]
pub struct Input {
    buffer: Vec<InputValue>,
    position: usize,
}

impl Input {
    pub fn new() -> Self {
        Self {
            buffer: Vec::new(),
            position: 0,
        }
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

    /// Clear all inputs and reset position
    pub fn clear(&mut self) {
        self.buffer.clear();
        self.position = 0;
    }

    /// Reset position to start (re-read inputs)
    pub fn rewind(&mut self) {
        self.position = 0;
    }

    /// Get remaining input count
    pub fn remaining(&self) -> usize {
        self.buffer.len().saturating_sub(self.position)
    }

    /// Check if there are more inputs available
    pub fn has_input(&self) -> bool {
        self.position < self.buffer.len()
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

    pub fn read_number(&mut self) -> Result<i32, VmError> {
        self.read().ok_or(VmError::InvalidInput)
    }

    pub fn read_char(&mut self) -> Result<i32, VmError> {
        self.read().ok_or(VmError::InvalidInput)
    }

    pub fn read_char_as_char(&mut self) -> Option<char> {
        self.read().and_then(|v| char::from_u32(v as u32))
    }
}

#[derive(Debug, Clone)]
pub struct Output {
    buffer: Vec<OutputValue>,
}

impl Output {
    pub fn new() -> Self {
        Self { buffer: Vec::new() }
    }

    pub fn write(&mut self, value: i32) {
        self.buffer.push(OutputValue::Number(value));
    }

    pub fn write_number(&mut self, value: i32) {
        self.buffer.push(OutputValue::Number(value));
    }

    pub fn write_char(&mut self, value: i32) {
        self.buffer.push(OutputValue::Char(value));
    }

    pub fn write_char_from_char(&mut self, c: char) {
        self.buffer.push(OutputValue::Char(c as i32));
    }

    pub fn read(&self) -> Vec<i32> {
        self.buffer.iter().map(|v| match v {
            OutputValue::Number(n) => *n,
            OutputValue::Char(c) => *c,
        }).collect()
    }

    pub fn read_string(&self) -> String {
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

    pub fn clear(&mut self) {
        self.buffer.clear();
    }
}

impl Default for Input {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for Output {
    fn default() -> Self {
        Self::new()
    }
}
