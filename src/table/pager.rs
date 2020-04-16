//! # Pager
//! 
//! Interface to load, hold and store pages into a file

use crate::constants::{PAGE_SIZE, TABLE_MAX_PAGES};
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};

/// A struct to hold all the pages and file metadata
pub struct Pager {
    pub file: File,
    pub file_length: u64,
    pub pages: Vec<Vec<u8>>,
}

impl Pager {
    /// Opens a file to load the pages from,
    /// if the file is not present, a new file is created.
    /// Finally returns a `Pager` struct with relevant data
    ///
    /// # Arguments
    ///
    /// * `filename` - A string slice holding the file name
    ///
    /// # Example
    ///
    /// ```
    /// use crate table::pager::Pager;
    /// let pager = Pager::open("test.db");
    /// ```
    ///
    /// # Panics
    ///
    /// Function might panic if there is some problem in creating or opening a file
    pub fn open(filename: &str) -> Self {
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

    /// Gets the page corresponding to the `page_num`
    /// 
    /// # Arguments
    /// 
    /// * `page_num` - The index of the page to be loaded
    pub fn get_page(&mut self, page_num: u32) {
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

                if self.file.read_exact(page.as_mut_slice()).is_err() {
                    println!("Error reading file. {}", page.len());
                    std::process::exit(1);
                }
                self.pages[page_num as usize].extend_from_slice(page.as_slice());
            }
        }
    }

    /// Writes the page with given page number to the file on disk
    /// 
    /// # Arguments
    /// 
    /// * `page_num` - The index of the page to be written to the disk
    pub fn flush(&mut self, page_num: u32) {
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
