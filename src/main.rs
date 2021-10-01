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
    num_bits_in_first_byte: u8,
    checksum: u8,
    decompressed_data_size: u16,
    compressed_data_size: u16,
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
                num_bits_in_first_byte,
                checksum,
                decompressed_data_size,
                compressed_data_size,
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

fn main() {
    let maindata = fs::read("main.dat").expect("give me main.dat");

    println!("read {} bytes\n", maindata.len());
    let mut offset = 0;

    loop {
        let (header, o) = Header::read(&maindata, offset).expect("bad file");
        offset = o + (header.compressed_data_size as usize) - 10;

        println!("found header:\n\n{}\n", header);

        match offset.cmp(&maindata.len()) {
            Ordering::Equal => break,
            Ordering::Greater => panic!("bad file"),
            Ordering::Less => continue,
        };
    }
}
