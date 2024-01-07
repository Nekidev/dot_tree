use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

mod utils;

// NEKOTREE
const FILE_IDENTIFIER: [u8; 8] = [0x4e, 0x45, 0x4b, 0x4f, 0x54, 0x52, 0x45, 0x45];
const FORMAT_VERSION: [u8; 2] = [0_u8, 0_u8];

#[derive(Debug)]
pub enum TreeFileError {
    FileNotOpened,
    FileHasContents,
    MissingHeaders,
    InvalidIdentifier,
    UnsupportedFormatVersion,
}

#[derive(PartialEq, Debug, EnumIter)]
pub enum Feature {
    Disabling,
}

#[derive(Debug)]
pub struct Tree {
    file: File,
    features: Vec<Feature>,
    subitems: Vec<u32>,
}

impl Tree {
    pub fn open(file_path: &'static str) -> Result<Self, TreeFileError> {
        let mut features: Vec<Feature> = vec![];
        let mut subitems: Vec<u32> = vec![];

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

        if file_headers[8..10] != FORMAT_VERSION {
            return Err(TreeFileError::UnsupportedFormatVersion);
        };

        let feature_bits = utils::bytes_to_bits(&file_headers[10..12]);
        let mut i = 0;
        for feature in Feature::iter() {
            if feature_bits[i] {
                features.push(feature);
            }
            i += 1;
        }

        let subitem_count = utils::u8_array_to_u32(&match &file_headers[12..16] {
            [a, b, c, d] => [*a, *b, *c, *d],
            _ => panic!("Slice does not have a length of 4"),
        });
        for _ in 0..subitem_count {
            let mut subitem_bytes = [0_u8; 4];
            match file.read_exact(&mut subitem_bytes) {
                Ok(_) => (),
                Err(_) => return Err(TreeFileError::MissingHeaders),
            };
            subitems.push(utils::u8_array_to_u32(&subitem_bytes));
        }

        Ok(Self {
            file,
            features,
            subitems,
        })
    }

    pub fn create(
        file_path: &'static str,
        features: Vec<Feature>,
        subitems: Vec<u32>,
    ) -> Result<Self, TreeFileError> {
        let mut file = match OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .append(true)
            .open(&file_path)
        {
            Ok(file) => file,
            Err(_) => return Err(TreeFileError::FileNotOpened),
        };

        let mut file_buffer = [0_u8; 1];
        match file.read_exact(&mut file_buffer) {
            Ok(_) => {
                return Err(TreeFileError::FileHasContents);
            }
            Err(_) => (),
        };

        file.write(&FILE_IDENTIFIER).unwrap();
        file.write(&FORMAT_VERSION).unwrap();

        let mut feature_bits = vec![features.contains(&Feature::Disabling)];
        feature_bits.extend(vec![false; 16 - feature_bits.len()]); // Align to 2 bytes
        file.write(&utils::bits_to_bytes(&feature_bits)).unwrap();

        file.write(&utils::u32_to_u8_array(subitems.len() as u32))
            .unwrap();

        for subitem in &subitems {
            file.write(&utils::u32_to_u8_array(*subitem)).unwrap();
        }

        Ok(Self {
            file,
            features,
            subitems,
        })
    }
}
