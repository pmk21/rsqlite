//! This file specifies the various constants used across the files.

/// Set page size to be used internally.
/// 4KB is the most common page size
pub const PAGE_SIZE: u32 = 4096;

/// Maximum amount of pages a table can hold or store at a time
pub const TABLE_MAX_PAGES: u32 = 100;

/// Number of rows that can fit into a page
pub const ROWS_PER_PAGE: u32 = PAGE_SIZE / ROW_SIZE;

/// Maximum rows a table can hold or store
pub const TABLE_MAX_ROWS: u32 = ROWS_PER_PAGE * TABLE_MAX_PAGES;

/// Size of the id field in bytes
pub const ID_SIZE: usize = 4;

/// Size of the username field in bytes
pub const USERNAME_SIZE: usize = 32;

/// Size of the email field in bytes
pub const EMAIL_SIZE: usize = 255;

// Since all the fields are converted to bytes and stored in a single byte array.
// The three fields have to be deserialized from different offsets in the byte array.
// The following constants specify those offsets.

/// The offset in the byte array where bytes of the id field start
pub const ID_OFFSET: usize = 0;

/// The offset in the byte array where bytes of the username field start
pub const USERNAME_OFFSET: usize = ID_OFFSET + ID_SIZE;

/// The offset in the byte array where bytes of the email field start
pub const EMAIL_OFFSET: usize = USERNAME_OFFSET + USERNAME_SIZE;

/// Total amount in bytes that a row will occupy in memory
pub const ROW_SIZE: u32 = (ID_SIZE + USERNAME_SIZE + EMAIL_SIZE) as u32;
