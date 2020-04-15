use crate::buffer::InputBuffer;
use crate::constants::{EMAIL_SIZE, TABLE_MAX_ROWS, USERNAME_SIZE};
use crate::table::{deserialize_row, print_row, row_slot, serialize_row, Row, Table};
use std::str::FromStr;

pub mod statement;

use statement::{Statement, StatementType};

pub enum ExecuteResult {
    Success,
    TableFull,
}

pub enum MetaCommandResult {
    UnrecognizedCommand,
}

pub enum PrepareResult {
    Success,
    UnrecognizedStatement,
    SyntaxError,
    StringTooLong,
    NegativeID,
}

pub fn do_meta_command(input_buffer: &InputBuffer, table: &mut Table) -> MetaCommandResult {
    if input_buffer.buffer == ".exit" {
        table.db_close();
        std::process::exit(0);
    } else {
        MetaCommandResult::UnrecognizedCommand
    }
}

pub fn prepare_insert(args: &[&str], statement: &mut Statement) -> PrepareResult {
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

pub fn prepare_statement(input_buffer: &InputBuffer, statement: &mut Statement) -> PrepareResult {
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

pub fn execute_statement(statement: &Statement, table: &mut Table) -> ExecuteResult {
    match statement.stmt_type {
        StatementType::Insert => execute_insert(statement, table),
        StatementType::Select => execute_select(statement, table),
        StatementType::Empty => {
            println!("Empty statement");
            ExecuteResult::Success
        }
    }
}

pub fn execute_insert(statement: &Statement, table: &mut Table) -> ExecuteResult {
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

#[allow(unused_variables)]
pub fn execute_select(statement: &Statement, table: &mut Table) -> ExecuteResult {
    for i in 0..table.num_rows {
        let (page_num, byte_offset) = row_slot(table, i);
        print_row(&deserialize_row(table, page_num, byte_offset));
    }
    ExecuteResult::Success
}
