use std::env;
use std::io::{self, Write};
use std::str::FromStr;

mod buffer;
mod constants;
mod table;

use constants::*;
use buffer::InputBuffer;
use table::Table;

enum MetaCommandResult {
    UnrecognizedCommand,
}

enum PrepareResult {
    Success,
    UnrecognizedStatement,
    SyntaxError,
    StringTooLong,
    NegativeID,
}

enum StatementType {
    Insert,
    Select,
    Empty,
}

struct Statement {
    stmt_type: StatementType,
    row_to_insert: Row,
}

struct Row {
    id: u32,
    username: [u8; USERNAME_SIZE],
    email: [u8; EMAIL_SIZE],
}

enum ExecuteResult {
    Success,
    TableFull,
}

fn print_prompt() {
    print!("db > ");
    // TODO: Handle error gracefully
    io::stdout().flush().expect("Could not flush stdout");
}

fn do_meta_command(input_buffer: &InputBuffer, table: &mut Table) -> MetaCommandResult {
    if input_buffer.buffer == ".exit" {
        table.db_close();
        std::process::exit(0);
    } else {
        MetaCommandResult::UnrecognizedCommand
    }
}

fn prepare_insert(args: &[&str], statement: &mut Statement) -> PrepareResult {
    statement.row_to_insert.id = match FromStr::from_str(args[1]) {
        Ok(uint) => uint,
        Err(_) => return PrepareResult::NegativeID,
    };

    let ubytes = args[2].as_bytes();
    let ulen = ubytes.len();
    if ulen > USERNAME_SIZE {
        return PrepareResult::StringTooLong;
    }
    let mut username_bytes = [0u8; USERNAME_SIZE];
    username_bytes[0..ulen].copy_from_slice(args[2].as_bytes());
    statement.row_to_insert.username = username_bytes;

    let ebytes = args[3].as_bytes();
    let elen = ebytes.len();
    if elen > EMAIL_SIZE {
        return PrepareResult::StringTooLong;
    }
    let mut email_bytes = [0u8; EMAIL_SIZE];
    email_bytes[0..elen].copy_from_slice(args[3].as_bytes());
    statement.row_to_insert.email = email_bytes;

    PrepareResult::Success
}

fn prepare_statement(input_buffer: &InputBuffer, statement: &mut Statement) -> PrepareResult {
    if &input_buffer.buffer[0..6] == "insert" {
        statement.stmt_type = StatementType::Insert;

        let args = input_buffer.buffer.split(' ').collect::<Vec<&str>>();
        if args.len() < 4 {
            return PrepareResult::SyntaxError;
        } else {
            return prepare_insert(&args, statement);
        }
    }

    if &input_buffer.buffer[0..6] == "select" {
        statement.stmt_type = StatementType::Select;
        return PrepareResult::Success;
    }
    PrepareResult::UnrecognizedStatement
}

fn execute_statement(statement: &Statement, table: &mut Table) -> ExecuteResult {
    match statement.stmt_type {
        StatementType::Insert => {
            execute_insert(statement, table)
        }
        StatementType::Select => {
            execute_select(statement, table)
        }
        StatementType::Empty => {
            println!("Empty statement");
            ExecuteResult::Success
        }
    }
}

fn execute_insert(statement: &Statement, table: &mut Table) -> ExecuteResult {
    if table.num_rows >= TABLE_MAX_ROWS {
        return ExecuteResult::TableFull;
    }

    let row = Row {
        id: statement.row_to_insert.id,
        username: statement.row_to_insert.username,
        email: statement.row_to_insert.email,
    };

    let (page_num, _) = row_slot(table, table.num_rows);
    serialize_row(row, table, page_num);
    table.num_rows += 1;

    ExecuteResult::Success
}

fn row_slot(table: &mut Table, row_num: u32) -> (u32, u32) {
    let page_num = row_num / ROWS_PER_PAGE;
    let row_offset = row_num % ROWS_PER_PAGE;
    let byte_offset = row_offset * ROW_SIZE;
    table.pager.get_page(page_num);
    (page_num, byte_offset)
}

fn serialize_row(row: Row, table: &mut Table, page_num: u32) {
    let id_bytes = row.id.to_ne_bytes();
    let username_bytes = row.username;
    let email_bytes = row.email;
    table.pager.pages[page_num as usize].extend_from_slice(&id_bytes);
    table.pager.pages[page_num as usize].extend_from_slice(&username_bytes);
    table.pager.pages[page_num as usize].extend_from_slice(&email_bytes);
}

fn deserialize_row(table: &Table, page_num: u32, byte_offset: u32) -> Row {
    let offset = byte_offset as usize;
    let mut id_byte_arr = [0; 4];
    let id_bytes_slice =
        &table.pager.pages[page_num as usize][(offset + ID_OFFSET)..(offset + ID_OFFSET + ID_SIZE)];
    let username_bytes = &table.pager.pages[page_num as usize]
        [(offset + USERNAME_OFFSET)..(offset + USERNAME_OFFSET + USERNAME_SIZE)];
    let email_bytes = &table.pager.pages[page_num as usize]
        [(offset + EMAIL_OFFSET)..(offset + EMAIL_OFFSET + EMAIL_SIZE)];

    id_byte_arr.copy_from_slice(id_bytes_slice);
    let id = u32::from_ne_bytes(id_byte_arr);
    let mut username = [0u8; USERNAME_SIZE];
    username.copy_from_slice(username_bytes);
    let mut email = [0u8; EMAIL_SIZE];
    email.copy_from_slice(email_bytes);

    Row {
        id,
        username,
        email,
    }
}

#[allow(unused_variables)]
fn execute_select(statement: &Statement, table: &mut Table) -> ExecuteResult {
    for i in 0..table.num_rows {
        let (page_num, byte_offset) = row_slot(table, i);
        print_row(&deserialize_row(table, page_num, byte_offset));
    }
    ExecuteResult::Success
}

fn print_row(row: &Row) {
    println!(
        "({}, {}, {})",
        row.id,
        std::str::from_utf8(&row.username)
            .unwrap()
            .trim_end_matches(char::from(0)),
        std::str::from_utf8(&row.email)
            .unwrap()
            .trim_end_matches(char::from(0))
    );
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Must supply database filename.");
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

        let mut statement: Statement = Statement {
            stmt_type: StatementType::Empty,
            row_to_insert: Row {
                id: 0,
                username: [0u8; USERNAME_SIZE],
                email: [0u8; EMAIL_SIZE],
            },
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
