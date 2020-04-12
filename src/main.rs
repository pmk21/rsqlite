use std::io::{self, BufRead, Write};

struct InputBuffer {
    buffer: String,
    buffer_length: usize,
    input_length: usize,
}

impl InputBuffer {
    fn new() -> Self {
        InputBuffer {
            buffer: String::new(),
            buffer_length: 0,
            input_length: 0,
        }
    }

    fn read_input(&mut self) {
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

fn print_prompt() {
    print!("db > ");
    // TODO: Handle error gracefully
    io::stdout().flush().expect("Could not flush stdout");
}

fn main() {
    let mut input_buffer = InputBuffer::new();

    loop {
        print_prompt();
        input_buffer.read_input();

        if input_buffer.buffer == ".exit" {
            std::process::exit(0);
        } else {
            println!("Unrecognized command '{}'.", input_buffer.buffer);
        }
    }
}
