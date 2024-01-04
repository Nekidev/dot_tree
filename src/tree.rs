use std::fs::{File, OpenOptions};
use std::io::Read;

// NEKOTREE
const FILE_IDENTIFIER: [u8; 8] = [0x4e, 0x45, 0x4b, 0x4f, 0x54, 0x52, 0x45, 0x45];

#[derive(Debug)]
pub enum TreeFileError {
    FileNotOpened,
    MissingHeaders,
    InvalidIdentifier,
}

#[derive(Debug)]
pub struct Tree {
    file: File,
    pub hash_size: u64,
}

impl Tree {
    pub fn open(file_path: &str) -> Result<Self, TreeFileError> {
        let mut file = match OpenOptions::new().read(true).create(false).open(&file_path) {
            Ok(file) => file,
            Err(_) => return Err(TreeFileError::FileNotOpened),
        };

        let mut file_headers = [0u8; 16];
        match file.read_exact(&mut file_headers) {
            Ok(_) => (),
            Err(_) => return Err(TreeFileError::MissingHeaders),
        };

        if file_headers[0..8] != FILE_IDENTIFIER {
            return Err(TreeFileError::InvalidIdentifier);
        };

        let hash_size = u64::from_be_bytes((&file_headers[8..16]).try_into().unwrap());

        Ok(Self {
            file,
            hash_size,
        })
    }

    pub fn create(file_path: &str) -> Result<Self, TreeFileError> {
        Err(TreeFileError::FileNotOpened)
    }
}
