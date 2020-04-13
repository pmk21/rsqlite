#![allow(dead_code)]

use std::io::{self, BufRead, Write};
use std::str::FromStr;

const PAGE_SIZE: u32 = 4096;
const TABLE_MAX_PAGES: u32 = 100;
const ROWS_PER_PAGE: u32 = PAGE_SIZE / ROW_SIZE;
const TABLE_MAX_ROWS: u32 = ROWS_PER_PAGE * TABLE_MAX_PAGES;

const COLUMN_USERNAME_SIZE: u32 = 32;
const COLUMN_EMAIL_SIZE: u32 = 255;

const ID_SIZE: usize = 4;
const USERNAME_SIZE: usize = 32;
const EMAIL_SIZE: usize = 255;
const ID_OFFSET: usize = 0;
const USERNAME_OFFSET: usize = ID_OFFSET + ID_SIZE;
const EMAIL_OFFSET: usize = USERNAME_OFFSET + USERNAME_SIZE;
const ROW_SIZE: u32 = (ID_SIZE + USERNAME_SIZE + EMAIL_SIZE) as u32;

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
    SyntaxError,
    StringTooLong,
    NegativeID
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

struct Table {
    num_rows: u32,
    pages: Vec<Vec<u8>>,
}

enum ExecuteResult {
    Success,
    TableFull,
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

impl Table {
    fn new() -> Self {
        Table {
            pages: vec![vec![]; TABLE_MAX_PAGES as usize],
            num_rows: 0,
        }
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

fn prepare_insert(args: &Vec<&str>, statement: &mut Statement) -> PrepareResult {
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
            return execute_insert(statement, table);
        }
        StatementType::Select => {
            return execute_select(statement, &*table);
        }
        StatementType::Empty => {
            println!("Empty statement");
            return ExecuteResult::Success;
        }
    }
}

fn execute_insert(statement: &Statement, table: &mut Table) -> ExecuteResult {
    if table.num_rows >= TABLE_MAX_ROWS {
        return ExecuteResult::TableFull;
    }

    let row = Row {
        id: statement.row_to_insert.id,
        username: statement.row_to_insert.username.clone(),
        email: statement.row_to_insert.email.clone(),
    };

    let (page_num, _) = row_slot(table.num_rows);
    serialize_row(row, table, page_num);
    table.num_rows += 1;

    return ExecuteResult::Success;
}

fn row_slot(row_num: u32) -> (u32, u32) {
    let page_num = row_num / ROWS_PER_PAGE;
    let row_offset = row_num % ROWS_PER_PAGE;
    let byte_offset = row_offset * ROW_SIZE;
    (page_num, byte_offset)
}

fn serialize_row(row: Row, table: &mut Table, page_num: u32) {
    let id_bytes = row.id.to_ne_bytes();
    let username_bytes = row.username;
    let email_bytes = row.email;
    table.pages[page_num as usize].extend_from_slice(&id_bytes);
    table.pages[page_num as usize].extend_from_slice(&username_bytes);
    table.pages[page_num as usize].extend_from_slice(&email_bytes);
}

fn deserialize_row(table: &Table, page_num: u32, byte_offset: u32) -> Row {
    let offset = byte_offset as usize;
    let mut id_byte_arr = [0; 4];
    let id_bytes_slice =
        &table.pages[page_num as usize][(offset + ID_OFFSET)..(offset + ID_OFFSET + ID_SIZE)];
    let username_bytes = &table.pages[page_num as usize]
        [(offset + USERNAME_OFFSET)..(offset + USERNAME_OFFSET + USERNAME_SIZE)];
    let email_bytes = &table.pages[page_num as usize]
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
fn execute_select(statement: &Statement, table: &Table) -> ExecuteResult {
    for i in 0..table.num_rows {
        let (page_num, byte_offset) = row_slot(i);
        // println!("{} {}", page_num, byte_offset);
        print_row(&deserialize_row(table, page_num, byte_offset));
    }
    return ExecuteResult::Success;
}

fn print_row(row: &Row) {
    println!(
        "({}, {}, {})",
        row.id,
        std::str::from_utf8(&row.username).unwrap(),
        std::str::from_utf8(&row.email).unwrap()
    );
}

fn main() {
    let mut input_buffer = InputBuffer::new();
    let mut table = Table::new();

    loop {
        print_prompt();
        input_buffer.read_input();

        if input_buffer.buffer.len() == 0 {
            continue;
        }

        if input_buffer.buffer.chars().next().unwrap() == '.' {
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
            },
            PrepareResult::StringTooLong => {
                println!("String is too long.");
                continue;
            },
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
