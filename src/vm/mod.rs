//! # VM
//!
//! A very basic "vm" for SQL

use crate::buffer::InputBuffer;
use crate::constants::{EMAIL_SIZE, TABLE_MAX_ROWS, USERNAME_SIZE};
use crate::table::{Row, Table};
use std::str::FromStr;

pub mod statement;

use statement::{Statement, StatementType};

/// Enum to show the result of executing a statement
pub enum ExecuteResult {
    Success,
    TableFull,
}

/// Enum to show the result of meta commands
pub enum MetaCommandResult {
    UnrecognizedCommand,
}

/// Enum to show the result of processing/preparing an SQL statement
pub enum PrepareResult {
    Success,
    UnrecognizedStatement,
    SyntaxError,
    StringTooLong,
    NegativeID,
}

/// Helper function to run a meta command
///
/// # Arguments
///
/// * `input_buffer` - Buffer storing the user input from stdin
/// * `table` - A `Table` struct holding current data
pub fn do_meta_command(input_buffer: &InputBuffer, table: &mut Table) -> MetaCommandResult {
    if input_buffer.buffer == ".exit" {
        table.db_close();
        std::process::exit(0);
    } else {
        MetaCommandResult::UnrecognizedCommand
    }
}

/// Helper function to transform data for insertion into the database
///
/// # Arguments
///
/// * `args` - Data corresponding to the fields in a row of the table
/// * `statement` - A `Statement` struct holding the type of statement and data to be inserted
/// in the case of an insert statement
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

/// Helper function to process/prepare a SQL statement
///
/// # Arguments
///
/// * `input_buffer` - Buffer storing the user input from stdin
/// * `statement` - A `Statement` struct holding the type of statement and relevant data based on the type
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

/// Helper function to execute a SQL statement based on its type
///
/// # Arguments
///
/// * `statement` - A `Statement` struct holding the type of statement and relevant data based on the type
/// * `table` - A `Table` struct holding current data
pub fn execute_statement(statement: &Statement, table: &mut Table) -> ExecuteResult {
    match statement.stmt_type {
        StatementType::Insert => execute_insert(statement, table),
        StatementType::Select => execute_select(table),
        StatementType::Empty => {
            println!("Empty statement");
            ExecuteResult::Success
        }
    }
}

/// Helper function to execute a SQL insert statement
///
/// # Arguments
///
/// * `statement` - A `Statement` struct holding the type of statement and relevant data based on the type
/// * `table` - A `Table` struct holding current data
fn execute_insert(statement: &Statement, table: &mut Table) -> ExecuteResult {
    if table.num_rows >= TABLE_MAX_ROWS {
        return ExecuteResult::TableFull;
    }

    let row = Row {
        id: statement.row_to_insert.id,
        username: statement.row_to_insert.username,
        email: statement.row_to_insert.email,
    };

    let (page_num, _) = table.row_slot(table.num_rows);
    table.serialize_row(row, page_num);
    table.num_rows += 1;

    ExecuteResult::Success
}

/// Helper function to execute a SQL select statement
///
/// # Arguments
///
/// * `statement` - A `Statement` struct holding the type of statement and relevant data based on the type
/// * `table` - A `Table` struct holding current data
fn execute_select(table: &mut Table) -> ExecuteResult {
    for i in 0..table.num_rows {
        let (page_num, byte_offset) = table.row_slot(i);
        &table.deserialize_row(page_num, byte_offset).print_row();
    }
    ExecuteResult::Success
}
