use std::io::{self, BufRead, Write};

struct InputBuffer {
    buffer: String,
    buffer_length: usize,
    input_length: usize,
}

enum MetaCommandResult {
    Success,
    UnrecognizedCommand,
}

enum PrepareResult {
    Success,
    UnrecognizedStatement,
}

enum StatementType {
    Insert,
    Select,
    Empty,
}

struct Statement {
    stmt_type: StatementType,
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

fn do_meta_command(input_buffer: &InputBuffer) -> MetaCommandResult {
    if input_buffer.buffer == ".exit" {
        std::process::exit(0);
    } else {
        return MetaCommandResult::UnrecognizedCommand;
    }
}

fn prepare_statement(input_buffer: &InputBuffer, statement: &mut Statement) -> PrepareResult {
    if &input_buffer.buffer[0..6] == "insert" {
        statement.stmt_type = StatementType::Insert;
        return PrepareResult::Success;
    }

    if &input_buffer.buffer[0..6] == "select" {
        statement.stmt_type = StatementType::Select;
        return PrepareResult::Success;
    }
    PrepareResult::UnrecognizedStatement
}

fn execute_statement(statement: &Statement) {
    match statement.stmt_type {
        StatementType::Insert => {
            println!("This is where we would do an insert");
        },
        StatementType::Select => {
            println!("This is where we would do a select");
        },
        StatementType::Empty => {
            println!("Empty statement");
        }
    }
}

fn main() {
    let mut input_buffer = InputBuffer::new();

    loop {
        print_prompt();
        input_buffer.read_input();

        if &input_buffer.buffer[0..1] == "." {
            match do_meta_command(&input_buffer) {
                MetaCommandResult::Success => continue,
                MetaCommandResult::UnrecognizedCommand => {
                    println!("Unrecognized command '{}'.", input_buffer.buffer);
                    continue;
                }
            }
        }

        let mut statement: Statement = Statement {
            stmt_type: StatementType::Empty,
        };

        match prepare_statement(&input_buffer, &mut statement) {
            PrepareResult::Success => (),
            PrepareResult::UnrecognizedStatement => {
                println!(
                    "Unrecognized keyword at the start of '{}'.",
                    input_buffer.buffer
                );
                continue;
            }
        }

        execute_statement(&statement);
        println!("Executed");
    }
}
