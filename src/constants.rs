pub const PAGE_SIZE: u32 = 4096;
pub const TABLE_MAX_PAGES: u32 = 100;
pub const ROWS_PER_PAGE: u32 = PAGE_SIZE / ROW_SIZE;
pub const TABLE_MAX_ROWS: u32 = ROWS_PER_PAGE * TABLE_MAX_PAGES;

pub const ID_SIZE: usize = 4;
pub const USERNAME_SIZE: usize = 32;
pub const EMAIL_SIZE: usize = 255;
pub const ID_OFFSET: usize = 0;
pub const USERNAME_OFFSET: usize = ID_OFFSET + ID_SIZE;
pub const EMAIL_OFFSET: usize = USERNAME_OFFSET + USERNAME_SIZE;
pub const ROW_SIZE: u32 = (ID_SIZE + USERNAME_SIZE + EMAIL_SIZE) as u32;
