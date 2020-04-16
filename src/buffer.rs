//! # Buffer
//! 
//! A small interface to read user input from stdin.

use std::io::{self, BufRead};

/// Structure to hold the user input
pub struct InputBuffer {
    pub buffer: String,
}

impl InputBuffer {
    /// Returns a new InputBuffer which contains an empty buffer
    /// 
    /// # Example
    /// 
    /// ```
    /// use crate::buffer::InputBuffer;
    /// let input_buffer = InputBuffer::new();
    /// ```
    pub fn new() -> Self {
        InputBuffer {
            buffer: String::new(),
        }
    }

    /// Reads user input from stdin
    /// 
    /// # Example
    /// 
    /// ```
    /// use crate::buffer::InputBuffer;
    /// let input_buffer = InputBuffer::new();
    /// // Input from stdin present in input_buffer.buffer
    /// input_buffer.read_input();
    /// ```
    pub fn read_input(&mut self) {
        self.buffer.clear();
        let stdin = io::stdin();
        stdin
            .lock()
            .read_line(&mut self.buffer)
            .expect("Could not read from stdin");
        // TODO: Find better way to remove newline character
        self.buffer.pop();
    }
}
