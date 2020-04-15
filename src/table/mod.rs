use crate::constants::{
    EMAIL_OFFSET, EMAIL_SIZE, ID_OFFSET, ID_SIZE, ROWS_PER_PAGE, ROW_SIZE, USERNAME_OFFSET,
    USERNAME_SIZE,
};

pub mod pager;
use pager::Pager;

pub struct Table {
    pub num_rows: u32,
    pub pager: Pager,
}

impl Table {
    pub fn db_open(filename: &str) -> Self {
        let pager = Pager::open(filename);
        let num_rows = pager.file_length as u32 / ROW_SIZE;
        Table { pager, num_rows }
    }

    pub fn db_close(&mut self) {
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

    pub fn serialize_row(&mut self, row: Row, page_num: u32) {
        let id_bytes = row.id.to_ne_bytes();
        let username_bytes = row.username;
        let email_bytes = row.email;
        self.pager.pages[page_num as usize].extend_from_slice(&id_bytes);
        self.pager.pages[page_num as usize].extend_from_slice(&username_bytes);
        self.pager.pages[page_num as usize].extend_from_slice(&email_bytes);
    }

    pub fn deserialize_row(&self, page_num: u32, byte_offset: u32) -> Row {
        let offset = byte_offset as usize;
        let mut id_byte_arr = [0; 4];
        let id_bytes_slice = &self.pager.pages[page_num as usize]
            [(offset + ID_OFFSET)..(offset + ID_OFFSET + ID_SIZE)];
        let username_bytes = &self.pager.pages[page_num as usize]
            [(offset + USERNAME_OFFSET)..(offset + USERNAME_OFFSET + USERNAME_SIZE)];
        let email_bytes = &self.pager.pages[page_num as usize]
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
}

pub struct Row {
    pub id: u32,
    pub username: [u8; USERNAME_SIZE],
    pub email: [u8; EMAIL_SIZE],
}

impl Row {
    pub fn new() -> Self {
        Row {
            id: 0,
            username: [0u8; USERNAME_SIZE],
            email: [0u8; EMAIL_SIZE],
        }
    }
}

pub fn print_row(row: &Row) {
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

pub fn row_slot(table: &mut Table, row_num: u32) -> (u32, u32) {
    let page_num = row_num / ROWS_PER_PAGE;
    let row_offset = row_num % ROWS_PER_PAGE;
    let byte_offset = row_offset * ROW_SIZE;
    table.pager.get_page(page_num);
    (page_num, byte_offset)
}
