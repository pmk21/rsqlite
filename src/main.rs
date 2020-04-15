use std::env;
use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, Read, Seek, SeekFrom, Write};
use std::str::FromStr;

const PAGE_SIZE: u32 = 4096;
const TABLE_MAX_PAGES: u32 = 100;
const ROWS_PER_PAGE: u32 = PAGE_SIZE / ROW_SIZE;
const TABLE_MAX_ROWS: u32 = ROWS_PER_PAGE * TABLE_MAX_PAGES;

const ID_SIZE: usize = 4;
const USERNAME_SIZE: usize = 32;
const EMAIL_SIZE: usize = 255;
const ID_OFFSET: usize = 0;
const USERNAME_OFFSET: usize = ID_OFFSET + ID_SIZE;
const EMAIL_OFFSET: usize = USERNAME_OFFSET + USERNAME_SIZE;
const ROW_SIZE: u32 = (ID_SIZE + USERNAME_SIZE + EMAIL_SIZE) as u32;

struct InputBuffer {
    buffer: String,
}

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

struct Pager {
    file: File,
    file_length: u64,
    pages: Vec<Vec<u8>>,
}

struct Table {
    num_rows: u32,
    pager: Pager,
}

enum ExecuteResult {
    Success,
    TableFull,
}

impl InputBuffer {
    fn new() -> Self {
        InputBuffer {
            buffer: String::new(),
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
    fn db_open(filename: &str) -> Self {
        let pager = Pager::open(filename);
        let num_rows = pager.file_length as u32 / ROW_SIZE;
        Table { pager, num_rows }
    }

    fn db_close(&mut self) {
        let num_full_pages = self.num_rows / ROWS_PER_PAGE;

        for i in 0..num_full_pages {
            if self.pager.pages[i as usize].is_empty() {
                continue;
            }
            self.pager.flush(i);
        }

        // There may be a partial page to write to the end of the file
        // This should not be needed after we switch to a B-tree
        let num_add_rows = self.num_rows % ROWS_PER_PAGE;
        if num_add_rows > 0 {
            let page_num = num_full_pages;
            if !self.pager.pages[page_num as usize].is_empty() {
                self.pager.flush(page_num);
            }
        }

        if self.pager.file.sync_data().is_err() {
            println!("Error closing db file.");
            std::process::exit(1);
        }
    }
}

impl Pager {
    fn open(filename: &str) -> Self {
        let mut file = OpenOptions::new()
            .write(true)
            .read(true)
            .create(true)
            .open(filename)
            .unwrap();
        let file_length = file.seek(SeekFrom::End(0)).unwrap();

        Pager {
            file,
            file_length,
            pages: vec![vec![]; TABLE_MAX_PAGES as usize],
        }
    }
    fn get_page(&mut self, page_num: u32) {
        if page_num > TABLE_MAX_PAGES {
            println!(
                "Tried to fetch page number out of bounds. {} > {}",
                page_num, TABLE_MAX_PAGES
            );
            std::process::exit(1);
        }

        if self.pages[page_num as usize].is_empty() {
            // Cache miss. Load from file
            let mut num_pages = self.file_length / PAGE_SIZE as u64;

            // We might save a partial page at the end of the file
            if self.file_length % PAGE_SIZE as u64 > 0 {
                num_pages += 1;
            }

            if page_num as u64 <= num_pages {
                if self
                    .file
                    .seek(SeekFrom::Start((page_num * PAGE_SIZE) as u64))
                    .is_err()
                {
                    println!("Error seeking file.");
                    std::process::exit(1);
                }
                let buf_size: usize = if ((page_num * PAGE_SIZE) as u64) <= self.file_length {
                    (self.file_length - (page_num * PAGE_SIZE) as u64) as usize
                } else {
                    PAGE_SIZE as usize
                };

                let mut page: Vec<u8> = vec![0; buf_size];
                // TODO: Better error handling mechanism
                if self.file.read_exact(page.as_mut_slice()).is_err() {
                    println!("Error reading file. {}", page.len());
                    std::process::exit(1);
                }
                self.pages[page_num as usize].extend_from_slice(page.as_slice());
            }
        }
    }

    fn flush(&mut self, page_num: u32) {
        if self.pages[page_num as usize].is_empty() {
            println!("Tried to flush null page");
            std::process::exit(1);
        }

        if self
            .file
            .seek(SeekFrom::Start((page_num * PAGE_SIZE) as u64))
            .is_err()
        {
            println!("Error seeking.");
            std::process::exit(1);
        }

        let drained_vec: Vec<u8> = self.pages[page_num as usize].drain(..).collect();
        self.pages[page_num as usize].shrink_to_fit();

        if self.file.write_all(drained_vec.as_ref()).is_err() {
            println!("Error writing.");
            std::process::exit(1);
        }
    }
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
