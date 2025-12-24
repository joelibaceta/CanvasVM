use crate::error::VmError;

/// Tipo de salida para distinguir números de caracteres
#[derive(Debug, Clone, Copy)]
pub enum OutputValue {
    Number(i32),
    Char(i32),
}

/// Sistema de entrada/salida para la VM
#[derive(Debug, Clone)]
pub struct Input {
    buffer: Vec<i32>,
    position: usize,
}

impl Input {
    pub fn new() -> Self {
        Self {
            buffer: Vec::new(),
            position: 0,
        }
    }

    pub fn write(&mut self, value: i32) {
        self.buffer.push(value);
    }

    pub fn read(&mut self) -> Option<i32> {
        if self.position < self.buffer.len() {
            let value = self.buffer[self.position];
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
