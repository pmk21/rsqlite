use crate::constants::{ROWS_PER_PAGE, ROW_SIZE};

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
}
