use anyhow::*;
use std::{cmp::Ordering, fmt};

use super::bitstream;

fn read_byte(buffer: &Vec<u8>, offset: usize) -> Result<(u8, usize)> {
    return Ok((
        *buffer
            .get(offset)
            .ok_or(anyhow!("offset {} out of bounds", offset))?,
        offset + 1,
    ));
}

fn read_word(buffer: &Vec<u8>, offset: usize) -> Result<(u16, usize)> {
    return Ok((
        (read_byte(buffer, offset)?.0 as u16) << 8 | (read_byte(buffer, offset + 1)?.0 as u16),
        offset + 2,
    ));
}

pub struct Header {
    pub num_bits_in_first_byte: usize,
    pub checksum: u8,
    pub decompressed_data_size: usize,
    pub compressed_data_size: usize,
}

pub struct Section {
    pub header: Header,
    pub data: Vec<u8>,
}

pub struct Content {
    pub sections: Vec<Section>,
}

impl fmt::Display for Header {
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

pub fn read_header(buffer: &Vec<u8>, offset: usize) -> Result<(Header, usize)> {
    let (num_bits_in_first_byte, offset) = read_byte(&buffer, offset)?;
    let (checksum, offset) = read_byte(&buffer, offset)?;

    let offset = offset + 2;
    let (decompressed_data_size, offset) = read_word(&buffer, offset)?;

    let offset = offset + 2;
    let (compressed_data_size, offset) = read_word(&buffer, offset)?;

    if compressed_data_size < 10 {
        bail!("compressed data size {} too small", compressed_data_size);
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

pub fn calculate_checksum(header: &Header, buffer: &Vec<u8>, offset: usize) -> Result<u8> {
    let mut checksum: u8 = 0;

    if offset + header.compressed_data_size - 10 > buffer.len() {
        bail!("not enough data in buffer");
    }

    let compressed_section = buffer
        .get(offset..header.compressed_data_size - 10 + offset)
        .ok_or(anyhow!("not enough data in buffer"))?;

    for value in compressed_section {
        checksum ^= value;
    }

    Ok(checksum)
}

pub fn decompress_section(
    bitstream: &mut bitstream::Bitstream,
    target: &mut Vec<u8>,
) -> Result<()> {
    target.reverse();

    while bitstream.remaining() > 0 {
        let first_bit: u8 = bitstream.consume(1)?;

        let opcode = if first_bit == 0 {
            (first_bit << 1) | bitstream.consume(1)?
        } else {
            (first_bit << 2) | bitstream.consume(2)?
        };

        match opcode {
            7 => {
                let count = bitstream.consume(8)? + 9;

                for _ in 0..count {
                    target.push(bitstream.consume(8)?)
                }
            }
            1 => {
                let offset = bitstream.consume(8)?;

                for _ in 0..2 {
                    target.push(
                        *target
                            .get(target.len() - 1 - offset as usize)
                            .ok_or(anyhow!("opcode 1: reference out of bounds"))?,
                    )
                }
            }
            4 => {
                let offset: usize =
                    ((bitstream.consume(8)? as usize) << 1) | (bitstream.consume(1)? as usize);

                for _ in 0..3 {
                    target.push(
                        *target
                            .get(target.len() - 1 - offset as usize)
                            .ok_or(anyhow!("opcode 4: reference out of bounds"))?,
                    )
                }
            }
            5 => {
                let offset: usize =
                    ((bitstream.consume(8)? as usize) << 2) | (bitstream.consume(2)? as usize);

                for _ in 0..4 {
                    target.push(
                        *target
                            .get(target.len() - 1 - offset as usize)
                            .ok_or(anyhow!("opcode 5: reference out of bounds"))?,
                    )
                }
            }
            6 => {
                let block_size = bitstream.consume(8)? as usize + 1;
                let offset: usize =
                    ((bitstream.consume(8)? as usize) << 4) | (bitstream.consume(4)? as usize);

                for _ in 0..block_size {
                    target.push(
                        *target
                            .get(target.len() - 1 - offset as usize)
                            .ok_or(anyhow!("opcode 6: reference out of bounds"))?,
                    )
                }
            }
            0 => {
                let count = bitstream.consume(3)? + 1;

                for _ in 0..count {
                    target.push(bitstream.consume(8)?)
                }
            }

            _ => bail!("bad chunk type {}", opcode),
        }
    }

    return Ok(target.reverse());
}

pub fn parse(data: &Vec<u8>) -> Result<Content> {
    let mut offset = 0;
    let mut sections: Vec<Section> = Vec::new();

    loop {
        let (header, o) = read_header(data, offset)?;
        let checksum = calculate_checksum(&header, data, o)?;

        if checksum != header.checksum {
            bail!("checksum mismatch");
        }

        let mut section_data: Vec<u8> = Vec::with_capacity(header.decompressed_data_size);
        decompress_section(
            &mut bitstream::Bitstream::create(
                data.get(o..o + header.compressed_data_size - 10)
                    .ok_or(anyhow!("out of bounds decompressing section"))?
                    .to_vec(),
                header.num_bits_in_first_byte,
            ),
            &mut section_data,
        )?;

        if section_data.len() != header.decompressed_data_size {
            bail!("decompressed section does not match header");
        }

        offset = o + header.compressed_data_size - 10;
        sections.push(Section {
            header,
            data: section_data,
        });

        match offset.cmp(&data.len()) {
            Ordering::Equal => break,
            Ordering::Greater => panic!("bad file"),
            Ordering::Less => continue,
        };
    }

    return Ok(Content { sections });
}

#[cfg(test)]
mod test_decompress_section {
    use super::bitstream::Bitstream;
    use super::decompress_section;

    #[test]
    fn test_op7() {
        let mut bitstream = Bitstream::create_from_example(
            r#"111 00000000 00000001 00000010 00000011 00000100 00000101 00000110
            00000111 00001000 00001001"#,
        );

        let mut target: Vec<u8> = Vec::new();

        decompress_section(&mut bitstream, &mut target).expect("decompress failed");

        assert_eq!(
            target,
            vec![0x09, 0x08, 0x07, 0x06, 0x05, 0x04, 0x03, 0x02, 0x01]
        );
    }

    #[test]
    fn test_op1_example1() {
        let mut bitstream = Bitstream::create_from_example("01 00000001");
        let mut target: Vec<u8> = vec![0x01, 0x02, 0x03, 0x09, 0x07];

        decompress_section(&mut bitstream, &mut target).expect("decompress failed");

        assert_eq!(target, vec![0x01, 0x02, 0x01, 0x02, 0x03, 0x09, 0x07]);
    }

    #[test]
    fn test_op1_example2() {
        let mut bitstream = Bitstream::create_from_example("01 00000100");
        let mut target: Vec<u8> = vec![0x01, 0x02, 0x03, 0x09, 0x07];

        decompress_section(&mut bitstream, &mut target).expect("decompress failed");

        assert_eq!(target, vec![0x09, 0x07, 0x01, 0x02, 0x03, 0x09, 0x07]);
    }

    #[test]
    fn test_op1_example3() {
        let mut bitstream = Bitstream::create_from_example("01 00000000");
        let mut target: Vec<u8> = vec![0x01, 0x02, 0x03, 0x09, 0x07];

        decompress_section(&mut bitstream, &mut target).expect("decompress failed");

        assert_eq!(target, vec![0x01, 0x01, 0x01, 0x02, 0x03, 0x09, 0x07]);
    }

    #[test]
    fn test_op4_example1() {
        let mut bitstream = Bitstream::create_from_example("100 000000011");
        let mut target: Vec<u8> = vec![0x01, 0x02, 0x03, 0x09, 0x07];

        decompress_section(&mut bitstream, &mut target).expect("decompress failed");

        assert_eq!(target, vec![0x02, 0x03, 0x09, 0x01, 0x02, 0x03, 0x09, 0x07]);
    }

    #[test]
    fn test_op4_example2() {
        let mut bitstream = Bitstream::create_from_example("100 000000001");
        let mut target: Vec<u8> = vec![0x01, 0x02, 0x03, 0x09, 0x07];

        decompress_section(&mut bitstream, &mut target).expect("decompress failed");

        assert_eq!(target, vec![0x02, 0x01, 0x02, 0x01, 0x02, 0x03, 0x09, 0x07]);
    }

    #[test]
    fn test_op4_example3() {
        let mut bitstream = Bitstream::create_from_example("100 000000000");
        let mut target: Vec<u8> = vec![0x01, 0x02, 0x03, 0x09, 0x07];

        decompress_section(&mut bitstream, &mut target).expect("decompress failed");

        assert_eq!(target, vec![0x01, 0x01, 0x01, 0x01, 0x02, 0x03, 0x09, 0x07]);
    }
}
