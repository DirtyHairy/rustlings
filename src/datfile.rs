use crate::bitstream;
use std::fmt::Display;

fn read_byte(buffer: &Vec<u8>, offset: usize) -> Result<(u8, usize), ()> {
    if offset >= buffer.len() {
        return Err(());
    }

    return Ok((buffer[offset], offset + 1));
}

fn read_word(buffer: &Vec<u8>, offset: usize) -> Result<(u16, usize), ()> {
    if offset + 1 >= buffer.len() {
        return Err(());
    }

    return Ok((
        (buffer[offset] as u16) << 8 | (buffer[offset + 1] as u16),
        offset + 2,
    ));
}

pub struct Header {
    pub num_bits_in_first_byte: usize,
    pub checksum: u8,
    pub decompressed_data_size: usize,
    pub compressed_data_size: usize,
}

impl Header {
    pub fn read(buffer: &Vec<u8>, offset: usize) -> Result<(Self, usize), ()> {
        let (num_bits_in_first_byte, offset) = read_byte(&buffer, offset)?;
        let (checksum, offset) = read_byte(&buffer, offset)?;

        let offset = offset + 2;
        let (decompressed_data_size, offset) = read_word(&buffer, offset)?;

        let offset = offset + 2;
        let (compressed_data_size, offset) = read_word(&buffer, offset)?;

        if compressed_data_size < 10 {
            return Err(());
        }

        Ok((
            Header {
                num_bits_in_first_byte: num_bits_in_first_byte as usize,
                checksum,
                decompressed_data_size: decompressed_data_size as usize,
                compressed_data_size: compressed_data_size as usize,
            },
            offset,
        ))
    }
}

impl Display for Header {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            r#"bits in first byte:       {}
checksum:                 {}
decompressed size:        {}
compressed size:          {}"#,
            self.num_bits_in_first_byte,
            self.checksum,
            self.decompressed_data_size,
            self.compressed_data_size
        )
    }
}

pub fn calculate_checksum(header: &Header, buffer: &Vec<u8>, offset: usize) -> Result<u8, ()> {
    let mut checksum: u8 = 0;

    if offset + header.compressed_data_size - 10 > buffer.len() {
        return Err(());
    }

    for value in buffer[offset..header.compressed_data_size - 10 + offset].iter() {
        checksum ^= value;
    }

    Ok(checksum)
}

pub fn decompress_section(bitstream: &mut bitstream::Bitstream, target: &mut Vec<u8>) {
    target.reverse();

    while bitstream.remaining() > 0 {
        let first_bit: u8 = bitstream.consume(1);

        let chunk_type = if first_bit == 0 {
            (first_bit << 1) | bitstream.consume(1)
        } else {
            (first_bit << 2) | bitstream.consume(2)
        };

        match chunk_type {
            7 => {
                let count = bitstream.consume(8) + 9;

                for _ in 0..count {
                    target.push(bitstream.consume(8))
                }
            }
            1 => {
                let offset = bitstream.consume(8);

                for _ in 0..2 {
                    target.push(target[target.len() - 1 - offset as usize])
                }
            }
            4 => {
                let offset: usize =
                    ((bitstream.consume(8) as usize) << 1) | (bitstream.consume(1) as usize);

                for _ in 0..3 {
                    target.push(target[target.len() - 1 - offset])
                }
            }
            5 => {
                let offset: usize =
                    ((bitstream.consume(8) as usize) << 2) | (bitstream.consume(2) as usize);

                for _ in 0..4 {
                    target.push(target[target.len() - 1 - offset])
                }
            }
            6 => {
                let block_size = bitstream.consume(8) as usize + 1;
                let offset: usize =
                    ((bitstream.consume(8) as usize) << 4) | (bitstream.consume(4) as usize);

                for _ in 0..block_size {
                    target.push(target[target.len() - 1 - offset])
                }
            }
            0 => {
                let count = bitstream.consume(3) + 1;

                for _ in 0..count {
                    target.push(bitstream.consume(8))
                }
            }
            _ => panic!("bad chunk type {}", chunk_type),
        }
    }

    target.reverse()
}
#[cfg(test)]
mod test_decompress_section {
    use crate::{bitstream::Bitstream, datfile::decompress_section};

    #[test]
    fn test_op7() {
        let mut bitstream = Bitstream::create_from_example(
            r#"111 00000000 00000001 00000010 00000011 00000100 00000101 00000110
            00000111 00001000 00001001"#,
        );

        let mut target: Vec<u8> = Vec::new();

        decompress_section(&mut bitstream, &mut target);

        assert_eq!(
            target,
            vec![0x09, 0x08, 0x07, 0x06, 0x05, 0x04, 0x03, 0x02, 0x01]
        );
    }

    #[test]
    fn test_op1_example1() {
        let mut bitstream = Bitstream::create_from_example("01 00000001");
        let mut target: Vec<u8> = vec![0x01, 0x02, 0x03, 0x09, 0x07];

        decompress_section(&mut bitstream, &mut target);

        assert_eq!(target, vec![0x01, 0x02, 0x01, 0x02, 0x03, 0x09, 0x07]);
    }

    #[test]
    fn test_op1_example2() {
        let mut bitstream = Bitstream::create_from_example("01 00000100");
        let mut target: Vec<u8> = vec![0x01, 0x02, 0x03, 0x09, 0x07];

        decompress_section(&mut bitstream, &mut target);

        assert_eq!(target, vec![0x09, 0x07, 0x01, 0x02, 0x03, 0x09, 0x07]);
    }

    #[test]
    fn test_op1_example3() {
        let mut bitstream = Bitstream::create_from_example("01 00000000");
        let mut target: Vec<u8> = vec![0x01, 0x02, 0x03, 0x09, 0x07];

        decompress_section(&mut bitstream, &mut target);

        assert_eq!(target, vec![0x01, 0x01, 0x01, 0x02, 0x03, 0x09, 0x07]);
    }

    #[test]
    fn test_op4_example1() {
        let mut bitstream = Bitstream::create_from_example("100 000000011");
        let mut target: Vec<u8> = vec![0x01, 0x02, 0x03, 0x09, 0x07];

        decompress_section(&mut bitstream, &mut target);

        assert_eq!(target, vec![0x02, 0x03, 0x09, 0x01, 0x02, 0x03, 0x09, 0x07]);
    }

    #[test]
    fn test_op4_example2() {
        let mut bitstream = Bitstream::create_from_example("100 000000001");
        let mut target: Vec<u8> = vec![0x01, 0x02, 0x03, 0x09, 0x07];

        decompress_section(&mut bitstream, &mut target);

        assert_eq!(target, vec![0x02, 0x01, 0x02, 0x01, 0x02, 0x03, 0x09, 0x07]);
    }

    #[test]
    fn test_op4_example3() {
        let mut bitstream = Bitstream::create_from_example("100 000000000");
        let mut target: Vec<u8> = vec![0x01, 0x02, 0x03, 0x09, 0x07];

        decompress_section(&mut bitstream, &mut target);

        assert_eq!(target, vec![0x01, 0x01, 0x01, 0x01, 0x02, 0x03, 0x09, 0x07]);
    }
}
