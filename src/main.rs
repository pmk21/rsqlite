use std::env;
use std::io::{self, Write};

mod buffer;
mod constants;
mod table;
mod vm;

use buffer::InputBuffer;
use table::Table;
use vm::statement::Statement;
use vm::{
    do_meta_command, execute_statement, prepare_statement, ExecuteResult, MetaCommandResult,
    PrepareResult,
};

/// Prints basic prompt onto stdout
fn print_prompt() {
    print!("db > ");
    io::stdout().flush().expect("Could not flush stdout");
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Must supply database filename.");
        println!("cargo run <filename>");
        std::process::exit(1);
    }

    let filename = &args[1];

    let mut input_buffer = InputBuffer::new();
    let mut table = Table::db_open(filename);

    loop {
        print_prompt();
        input_buffer.read_input();

        if input_buffer.buffer.is_empty() {
            continue;
        }

        if input_buffer.buffer.starts_with('.') {
            match do_meta_command(&input_buffer, &mut table) {
                MetaCommandResult::UnrecognizedCommand => {
                    println!("Unrecognized command '{}'.", input_buffer.buffer);
                    continue;
                }
            }
        }

        let mut statement: Statement = Statement::new();

        match prepare_statement(&input_buffer, &mut statement) {
            PrepareResult::Success => (),
            PrepareResult::UnrecognizedStatement => {
                println!(
                    "Unrecognized keyword at the start of '{}'.",
                    input_buffer.buffer
                );
                continue;
            }
            PrepareResult::SyntaxError => {
                println!("Syntax error. Could not parse statement.");
                continue;
            }
            PrepareResult::StringTooLong => {
                println!("String is too long.");
                continue;
            }
            PrepareResult::NegativeID => {
                println!("ID must be positive.");
                continue;
            }
        }

        match execute_statement(&statement, &mut table) {
            ExecuteResult::Success => {
                println!("Executed.");
            }
            ExecuteResult::TableFull => {
                println!("Error: Table full.");
            }
        }
    }
}
