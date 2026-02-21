use anyhow::{anyhow, Result};

pub struct Bitstream<'a> {
    buffer: &'a [u8],
    bit_index: usize,
    byte_index: usize,
    bits_in_first_byte: usize,
}

impl<'a> Bitstream<'a> {
    pub fn create(buffer: &'a [u8], bits_in_first_byte: usize) -> Self {
        Bitstream {
            buffer,
            bit_index: 0,
            byte_index: 0,
            bits_in_first_byte,
        }
    }

    #[cfg(test)]
    pub fn create_from_example(sample: &str) -> Self {
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
                _ => panic!("fix your sample"),
            }
        }

        if current_bit_count > 0 {
            buffer.push(current_byte)
        }

        Self::create(
            Box::leak(buffer.into_boxed_slice()),
            if current_bit_count == 0 {
                8
            } else {
                current_bit_count as usize
            },
        )
    }

    pub fn consume(&mut self, count: usize) -> Result<u8> {
        let mut value: u8 = 0;

        for _ in 0..count {
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

            let current_byte = *self
                .buffer
                .get(self.buffer.len() - self.byte_index - 1)
                .ok_or(anyhow!("cosume: out of bounds"))?;

            value <<= 1;
            value |= (current_byte >> self.bit_index) & 0x01;

            self.bit_index += 1;
        }

        Ok(value)
    }

    #[cfg(test)]
    pub fn consume_or_die(&mut self, count: usize) -> u8 {
        self.consume(count).expect("consume failed")
    }

    pub fn remaining(&self) -> usize {
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
mod test {
    use crate::game_data::file::encoding::bitstream::Bitstream;

    #[test]
    fn bitstream_example_3() {
        let mut bitstream = Bitstream::create(&[0x57, 0xa3], 6);

        assert_eq!(bitstream.remaining(), 14);

        assert_eq!(bitstream.consume_or_die(8), 0b11000111);
        assert_eq!(bitstream.remaining(), 6);

        assert_eq!(bitstream.consume_or_die(6), 0b101010);
        assert_eq!(bitstream.remaining(), 0);
    }

    #[test]
    fn bitstream_example_3_parts() {
        let mut bitstream = Bitstream::create(&[0x57, 0xa3], 6);

        assert_eq!(bitstream.remaining(), 14);

        assert_eq!(bitstream.consume_or_die(4), 0b1100);
        assert_eq!(bitstream.remaining(), 10);

        assert_eq!(bitstream.consume_or_die(5), 0b01111);
        assert_eq!(bitstream.remaining(), 5);

        assert_eq!(bitstream.consume_or_die(5), 0b01010);
        assert_eq!(bitstream.remaining(), 0);
    }

    #[test]
    fn bitstream_from_example_1() {
        let mut bitstream = Bitstream::create_from_example("10011");

        assert_eq!(bitstream.remaining(), 5);
        assert_eq!(bitstream.consume_or_die(5), 0b10011);
    }

    #[test]
    fn bitstream_from_example_2() {
        let mut bitstream = Bitstream::create_from_example("10 0   1 1");

        assert_eq!(bitstream.remaining(), 5);
        assert_eq!(bitstream.consume_or_die(5), 0b10011);
    }

    #[test]
    fn bitstream_from_example_3() {
        let mut bitstream = Bitstream::create_from_example("01 10010011");

        assert_eq!(bitstream.remaining(), 10);
        assert_eq!(bitstream.consume_or_die(2), 0b01);

        assert_eq!(bitstream.remaining(), 8);
        assert_eq!(bitstream.consume_or_die(8), 0b10010011);
    }
}
