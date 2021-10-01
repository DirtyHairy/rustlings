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

fn main() {
    let maindata = fs::read("main.dat").expect("give me main.dat");

    println!("read {} bytes\n", maindata.len());
    let offset = 0;

    let (num_bits_in_first_byte, offset) = read_byte(&maindata, offset).expect("bad file");
    let (checksum, offset) = read_byte(&maindata, offset).expect("bad file");

    let offset = offset + 2;
    let (decompressed_data_size, offset) = read_word(&maindata, offset).expect("bad file");

    let offset = offset + 2;
    let (compressed_data_size, _) = read_word(&maindata, offset).expect("bad file");

    println!("found header");
    println!("bits in first byte:       {}", num_bits_in_first_byte);
    println!("checksum:                 {}", checksum);
    println!("decompressed size:        {}", decompressed_data_size);
    println!("compressed size:          {}", compressed_data_size);
}
