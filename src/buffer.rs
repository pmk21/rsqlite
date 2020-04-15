use std::io::{self, BufRead};

pub struct InputBuffer {
    pub buffer: String,
}

impl InputBuffer {
    pub fn new() -> Self {
        InputBuffer {
            buffer: String::new(),
        }
    }

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
