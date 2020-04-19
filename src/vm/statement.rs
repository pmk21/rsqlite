//! # Statement
//!
//! An abstract interface for handling SQL statements

use crate::table::Row;

/// Enum to indicate the type of SQL statement
pub enum StatementType {
    Insert,
    Select,
    Empty,
}

/// Struct that holds the type of SQL statement and relevant data
pub struct Statement {
    pub stmt_type: StatementType,
    pub row_to_insert: Row,
}

impl Statement {
    /// Returns a `Statement` struct with an empty statement
    /// and empty row
    pub fn new() -> Self {
        Statement {
            stmt_type: StatementType::Empty,
            row_to_insert: Row::new(),
        }
    }
}
