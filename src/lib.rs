mod utils;
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

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
    MissingPermissions,
}

#[derive(Debug)]
pub enum NodeError {
    Disabled,
    Unexistent,
    InvalidIndex,
    InvalidSubitem,
    NodeAlreadyExists,
    MissingFeature,
}

#[derive(PartialEq, Debug, EnumIter)]
pub enum Feature {
    Disabling,
}

#[derive(Debug, PartialEq)]
pub enum TreeOpenMode {
    Read,
    ReadWrite,
}

#[derive(Debug)]
pub struct Tree {
    pub file: File,
    pub mode: TreeOpenMode,
    pub header_size: usize,
    pub features: Vec<Feature>,
    pub subitems: Vec<u32>,
}

#[derive(Debug)]
pub struct Node<'a> {
    tree: &'a mut Tree,
    pub position: u128,
    pub subitems: Vec<Vec<bool>>,
}

impl Tree {
    pub fn open(file_path: &'static str, mode: TreeOpenMode) -> Result<Self, TreeFileError> {
        let mut features: Vec<Feature> = vec![];
        let mut subitems: Vec<u32> = vec![];

        {
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
        }

        let file = match OpenOptions::new()
            .read(true)
            .write(mode == TreeOpenMode::ReadWrite)
            .open(&file_path)
        {
            Ok(file) => file,
            Err(_) => return Err(TreeFileError::FileNotOpened),
        };

        let header_size = 16 + (subitems.len() * 4) as usize;

        Ok(Self {
            file,
            mode,
            header_size,
            features,
            subitems,
        })
    }

    pub fn create(
        file_path: &'static str,
        mode: TreeOpenMode,
        features: Vec<Feature>,
        subitems: Vec<u32>,
    ) -> Result<Self, TreeFileError> {
        {
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
        }

        let file = OpenOptions::new()
            .read(true)
            .write(mode == TreeOpenMode::ReadWrite)
            .open(&file_path)
            .unwrap();

        let header_size = 16 + (subitems.len() * 4) as usize;

        Ok(Self {
            file,
            mode,
            header_size,
            features,
            subitems,
        })
    }

    pub fn node_size(&self) -> u32 {
        let mut size = 0;

        for subitem in &self.subitems {
            size += *subitem;
        }

        if self.features.contains(&Feature::Disabling) {
            size += 1;
        }

        size
    }

    pub fn nodes(&self) -> u64 {
        let tree_storage_size = match self.file.metadata() {
            Ok(metadata) => metadata.len() - self.header_size as u64,
            Err(_) => 0,
        };

        tree_storage_size * 8 / self.node_size() as u64
    }

    pub fn root(&mut self) -> Result<Node, NodeError> {
        self.node(0)
    }

    pub fn levels(&self) -> u32 {
        let nodes = self.nodes();

        if nodes != 0 {
            nodes.ilog2()
        } else {
            0
        }
    }

    pub fn node(&mut self, position: u128) -> Result<Node, NodeError> {
        let node_size = self.node_size() as f64;
        let nodes = self.nodes() as u128;

        let start_byte = ((self.header_size as f64) + (position as f64) * node_size / 8.0) as u64;
        let pad_l = (node_size * position as f64) % 8.0;
        let buf_size = ((pad_l + node_size) as u64).div_ceil(8);

        if position >= nodes as u128 {
            return Err(NodeError::Unexistent);
        };

        self.file.seek(SeekFrom::Start(start_byte as u64)).unwrap();

        let mut byte_buffer = vec![0_u8; buf_size as usize];

        match self.file.read_exact(&mut byte_buffer) {
            Ok(_) => (),
            Err(_) => return Err(NodeError::Unexistent),
        };

        let bit_buffer: Vec<bool> = utils::bytes_to_bits(&byte_buffer);

        let mut bits: Vec<bool> =
            bit_buffer[(pad_l as usize)..((pad_l + node_size) as usize)].to_vec();

        if self.features.contains(&Feature::Disabling) {
            if bits[0] == false {
                return Err(NodeError::Disabled);
            };

            bits.remove(0);
        };

        let mut subitems: Vec<Vec<bool>> = vec![];
        for subitem in &self.subitems {
            subitems.push(bits[0..*subitem as usize].to_vec());
            bits.drain(0..*subitem as usize);
        }

        Ok(Node {
            tree: self,
            position,
            subitems,
        })
    }

    pub fn set_node(
        &mut self,
        subitems: &Vec<Vec<bool>>,
        position: &u128,
        overwrite: bool,
        disabled: bool,
    ) -> Result<Node, NodeError> {
        let mut bits: Vec<bool> = vec![];

        if self.features.contains(&Feature::Disabling) {
            bits.push(!disabled);
        };

        let mut i: usize = 0;
        for subitem in &self.subitems {
            if subitems[i].len() != *subitem as usize {
                return Err(NodeError::InvalidSubitem);
            };
            i += 1;
        }

        bits.extend(subitems.concat());

        if !overwrite {
            match self.node(*position) {
                Ok(_) => return Err(NodeError::NodeAlreadyExists),
                Err(_) => (),
            };
        };

        let node_size = self.node_size() as u128;
        let nodes = self.nodes() as u128;
        if nodes < *position {
            // Must add empty (0s?) nodes before the position
            let _ = self.file.seek(SeekFrom::End(0_i64));
            let _ = self.file.write(&vec![
                0_u8;
                ((nodes - position) * node_size).div_ceil(8) as usize
            ]);
        };

        let start_byte = self.header_size as u128 + (position * node_size) / 8;
        let pad_l = (position * node_size) % 8;
        let buf_size = (pad_l + node_size).div_ceil(8);

        let mut byte_buffer = vec![0_u8; buf_size as usize];

        self.file.seek(SeekFrom::Start(start_byte as u64)).unwrap();
        match self.file.read_exact(&mut byte_buffer) {
            Ok(_) => (),
            Err(_) => {
                self.file
                    .seek(SeekFrom::Start(
                        (self.header_size as u128 + ((position * node_size as u128) / 8)) as u64,
                    ))
                    .unwrap();

                // Read only first byte to get the padding (and to avoid corrupting the previous node).
                byte_buffer = vec![0_u8];
                let _ = self.file.read_exact(&mut byte_buffer);
            }
        };

        let pad_l_bits = utils::bytes_to_bits(&byte_buffer)[..(pad_l as usize)].to_vec();
        let pad_r_bits =
            utils::bytes_to_bits(&byte_buffer)[((pad_l + node_size) % 8) as usize..].to_vec();

        let fragment_bits: Vec<bool> = vec![pad_l_bits, bits, pad_r_bits].concat();

        match self.file.seek(SeekFrom::Start(start_byte as u64)) {
            Ok(_) => (),
            Err(_) => return Err(NodeError::Unexistent),
        };
        match self.file.write(&utils::bits_to_bytes(&fragment_bits)) {
            Ok(_) => (),
            Err(_) => return Err(NodeError::Unexistent),
        };

        self.node(*position)
    }
}

impl Node<'_> {
    pub fn level(&self) -> u32 {
        if self.position != 0 {
            self.position.ilog2()
        } else {
            0
        }
    }

    pub fn parent(&mut self) -> Result<Node, NodeError> {
        if self.position == 0 {
            return Err(NodeError::Unexistent);
        };

        self.tree.node((self.position - 1) / 2)
    }

    pub fn child(&mut self, index: u8) -> Result<Node, NodeError> {
        if index > 1 {
            return Err(NodeError::InvalidIndex);
        }

        if self.position == 0 {
            self.tree.node(1 + index as u128)
        } else {
            self.tree.node(self.position * 2 + index as u128)
        }
    }

    pub fn is_leaf(&mut self) -> bool {
        self.child(0).is_err() && self.child(1).is_err()
    }

    pub fn add_child(
        &mut self,
        index: u8,
        subitems: Vec<Vec<bool>>,
        overwrite: bool,
    ) -> Result<Node, NodeError> {
        if index > 1 {
            return Err(NodeError::InvalidIndex);
        }

        if self.position == 0 {
            self.tree
                .set_node(&subitems, &(1 + index as u128), overwrite, true)
        } else {
            self.tree.set_node(
                &subitems,
                &(self.position * 2 + index as u128),
                overwrite,
                true,
            )
        }
    }

    pub fn disable(&mut self) -> Result<(), NodeError> {
        if !self.tree.features.contains(&Feature::Disabling) {
            return Err(NodeError::MissingFeature);
        };

        let _ = self
            .tree
            .set_node(&self.subitems, &self.position, true, true);

        Ok(())
    }

    pub fn enable(&mut self) -> Result<(), NodeError> {
        if !self.tree.features.contains(&Feature::Disabling) {
            return Err(NodeError::MissingFeature);
        };

        let _ = self
            .tree
            .set_node(&self.subitems, &self.position, true, false);

        Ok(())
    }

    pub fn update(&mut self, subitems: Vec<Vec<bool>>) -> Result<(), NodeError> {
        let _ = self.tree.set_node(&subitems, &self.position, false, false);
        self.subitems = subitems;

        Ok(())
    }

    pub fn refresh(&mut self) -> Result<Node, NodeError> {
        let node = match self.tree.node(self.position) {
            Ok(node) => node,
            Err(_) => return Err(NodeError::Unexistent),
        };

        self.position = node.position.clone();
        self.subitems = node.subitems.clone();

        Ok(node)
    }
}
