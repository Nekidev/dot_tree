pub fn bits_to_bytes(bits: &[bool]) -> Vec<u8> {
    let mut result = Vec::new();

    for chunk in bits.chunks(8) {
        let mut byte = 0u8;

        for (i, &bit) in chunk.iter().enumerate() {
            if bit {
                byte |= 1 << (7 - i);
            }
        }

        result.push(byte);
    }

    result
}

pub fn bytes_to_bits(bytes: &[u8]) -> Vec<bool> {
    let mut bits = Vec::new();

    for byte in bytes {
        for i in (0..8).rev() {
            bits.push((byte >> i) & 1 == 1);
        }
    }

    bits
}

pub fn u32_to_u8_array(number: u32) -> [u8; 4] {
    let byte1 = ((number >> 24) & 0xFF) as u8;
    let byte2 = ((number >> 16) & 0xFF) as u8;
    let byte3 = ((number >> 8) & 0xFF) as u8;
    let byte4 = (number & 0xFF) as u8;

    [byte1, byte2, byte3, byte4]
}

pub fn u8_array_to_u32(bytes: &[u8; 4]) -> u32 {
    let mut result: u32 = 0;

    for &byte in bytes.iter() {
        result = (result << 8) | (byte as u32);
    }

    result
}
