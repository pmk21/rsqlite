use crate::table::Row;

pub enum StatementType {
    Insert,
    Select,
    Empty,
}

pub struct Statement {
    pub stmt_type: StatementType,
    pub row_to_insert: Row,
}

impl Statement {
    pub fn new() -> Self {
        Statement {
            stmt_type: StatementType::Empty,
            row_to_insert: Row::new(),
        }
    }
}
