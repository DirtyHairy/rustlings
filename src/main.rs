use std::cmp::Ordering;
use std::fmt::Display;
use std::fs;

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

struct Header {
    num_bits_in_first_byte: usize,
    checksum: u8,
    decompressed_data_size: usize,
    compressed_data_size: usize,
}

impl Header {
    fn read(buffer: &Vec<u8>, offset: usize) -> Result<(Self, usize), ()> {
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

fn calculate_checksum(header: &Header, buffer: &Vec<u8>, offset: usize) -> Result<u8, ()> {
    let mut checksum: u8 = 0;

    if offset + header.compressed_data_size - 10 > buffer.len() {
        return Err(());
    }

    for value in buffer[offset..header.compressed_data_size - 10 + offset].iter() {
        checksum ^= value;
    }

    Ok(checksum)
}

struct Bitstream {
    buffer: Vec<u8>,
    bit_index: usize,
    byte_index: usize,
    bits_in_first_byte: usize,
}

impl Bitstream {
    fn create(buffer: Vec<u8>, bits_in_first_byte: usize) -> Self {
        Bitstream {
            buffer,
            bit_index: 0,
            byte_index: 0,
            bits_in_first_byte,
        }
    }

    #[cfg(test)]
    fn create_from_example(sample: &str) -> Self {
        let mut current_byte: u8 = 0;
        let mut current_bit_count: u8 = 0;

        let mut buffer: Vec<u8> = Vec::new();

        let mut gobble = |bit: u8| {
            current_byte <<= 1;
            current_byte |= bit;
            current_bit_count += 1;

            if current_bit_count == 8 {
                current_bit_count = 0;
                buffer.push(current_byte);

                current_byte = 0;
            }
        };

        for char in sample.chars().rev() {
            match char {
                '0' => gobble(0),
                '1' => gobble(1),
                ' ' | '\n' => continue,
                _ => panic!("fix yourr sample"),
            }
        }

        if current_bit_count > 0 {
            buffer.push(current_byte)
        }

        Self::create(
            buffer,
            if current_bit_count == 0 {
                8
            } else {
                current_bit_count as usize
            },
        )
    }

    fn consume(&mut self, count: usize) -> u8 {
        assert!(count <= self.remaining());

        let mut value: u8 = 0;

        for _ in 0..count {
            value <<= 1;
            value |=
                (self.buffer[self.buffer.len() - self.byte_index - 1] >> self.bit_index) & 0x01;

            self.bit_index += 1;
            if self.bit_index
                >= (if self.byte_index == 0 {
                    self.bits_in_first_byte
                } else {
                    8
                })
            {
                self.bit_index = 0;
                self.byte_index += 1;
            }
        }

        value
    }

    fn remaining(&self) -> usize {
        (self.buffer.len() * 8 - 8 + self.bits_in_first_byte)
            - self.bit_index
            - if self.byte_index > 0 {
                8 * self.byte_index - 8 + self.bits_in_first_byte
            } else {
                0
            }
    }
}

#[cfg(test)]
mod test_bitstream {
    use crate::Bitstream;

    #[test]
    fn bitstream_example_3() {
        let mut bitstream = Bitstream::create(vec![0x57, 0xa3], 6);

        assert_eq!(bitstream.remaining(), 14);

        assert_eq!(bitstream.consume(8), 0b11000111);
        assert_eq!(bitstream.remaining(), 6);

        assert_eq!(bitstream.consume(6), 0b101010);
        assert_eq!(bitstream.remaining(), 0);
    }

    #[test]
    fn bitstream_example_3_parts() {
        let mut bitstream = Bitstream::create(vec![0x57, 0xa3], 6);

        assert_eq!(bitstream.remaining(), 14);

        assert_eq!(bitstream.consume(4), 0b1100);
        assert_eq!(bitstream.remaining(), 10);

        assert_eq!(bitstream.consume(5), 0b01111);
        assert_eq!(bitstream.remaining(), 5);

        assert_eq!(bitstream.consume(5), 0b01010);
        assert_eq!(bitstream.remaining(), 0);
    }

    #[test]
    fn bitstream_from_example_1() {
        let mut bitstream = Bitstream::create_from_example("10011");

        assert_eq!(bitstream.remaining(), 5);
        assert_eq!(bitstream.consume(5), 0b10011);
    }

    #[test]
    fn bitstream_from_example_2() {
        let mut bitstream = Bitstream::create_from_example("10 0   1 1");

        assert_eq!(bitstream.remaining(), 5);
        assert_eq!(bitstream.consume(5), 0b10011);
    }

    #[test]
    fn bitstream_from_example_3() {
        let mut bitstream = Bitstream::create_from_example("01 10010011");

        assert_eq!(bitstream.remaining(), 10);
        assert_eq!(bitstream.consume(2), 0b01);

        assert_eq!(bitstream.remaining(), 8);
        assert_eq!(bitstream.consume(8), 0b10010011);
    }
}

fn decompress_section(bitstream: &mut Bitstream, target: &mut Vec<u8>) {
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
    use crate::{decompress_section, Bitstream};

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

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 2 {
        println!("usage: rustlings <main.dat>");
        return;
    }

    let maindata = fs::read(&args[1]).expect("give me main.dat");

    println!("read {} bytes\n", maindata.len());
    let mut offset = 0;

    let mut i = 0;

    loop {
        let (header, o) = Header::read(&maindata, offset).expect("bad file");

        println!("found header:\n{}", header);

        let checksum = calculate_checksum(&header, &maindata, o).expect("bad file");
        if checksum == header.checksum {
            println!("checksum OK!")
        } else {
            println!(
                "checksum mismatch, expected {}, got {}",
                header.checksum, checksum
            )
        }

        let mut decompressed_section: Vec<u8> = Vec::with_capacity(header.decompressed_data_size);

        decompress_section(
            &mut Bitstream::create(
                maindata[o..o + header.compressed_data_size - 10].to_vec(),
                header.num_bits_in_first_byte,
            ),
            &mut decompressed_section,
        );

        assert_eq!(decompressed_section.len(), header.decompressed_data_size);

        if i == 0 {
            for frame in 0..8 {
                for y in 0..10 {
                    for x in 0..16 {
                        let bit1 = (decompressed_section[((y * 16) + x) / 8 + frame * 40]
                            >> (7 - (((y * 16) + x) % 8)))
                            & 0x01;

                        let bit2 = (decompressed_section[((y * 16) + x) / 8 + frame * 40 + 20]
                            >> (7 - (((y * 16) + x) % 8)))
                            & 0x01;

                        let color = bit1 | (bit2 << 1);

                        print!(
                            "{}{}",
                            if (color) > 0 {
                                color.to_string()
                            } else {
                                String::from(" ")
                            },
                            if (color) > 0 {
                                color.to_string()
                            } else {
                                String::from(" ")
                            }
                        );
                    }

                    println!("");
                }

                println!("\n====\n");
            }
        }

        offset = o + header.compressed_data_size - 10;

        println!();
        i += 1;

        match offset.cmp(&maindata.len()) {
            Ordering::Equal => break,
            Ordering::Greater => panic!("bad file"),
            Ordering::Less => continue,
        };
    }
}
