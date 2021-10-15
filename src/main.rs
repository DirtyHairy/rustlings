mod bitstream;
mod datfile;
mod sprites;

use std::cmp::Ordering;
use std::fs;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 2 {
        println!("usage: rustlings <main.dat>");
        return;
    }

    let maindata = fs::read(&args[1]).expect("give me main.dat");

    println!("read {} bytes\n", maindata.len());
    let mut offset = 0;

    let mut maybe_walking_lemming: Option<sprites::Sprite> = None;
    let mut i = 0;

    loop {
        let (header, o) = datfile::Header::read(&maindata, offset).expect("bad file");

        println!("found header:\n{}", header);

        let checksum = datfile::calculate_checksum(&header, &maindata, o).expect("bad file");
        if checksum == header.checksum {
            println!("checksum OK!")
        } else {
            println!(
                "checksum mismatch, expected {}, got {}",
                header.checksum, checksum
            )
        }

        let mut decompressed_section: Vec<u8> = Vec::with_capacity(header.decompressed_data_size);

        datfile::decompress_section(
            &mut bitstream::Bitstream::create(
                maindata[o..o + header.compressed_data_size - 10].to_vec(),
                header.num_bits_in_first_byte,
            ),
            &mut decompressed_section,
        );

        assert_eq!(decompressed_section.len(), header.decompressed_data_size);

        if i == 0 {
            maybe_walking_lemming =
                sprites::Sprite::read_planar(8, 16, 10, 2, &decompressed_section).ok();
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

    print!("{}", maybe_walking_lemming.expect("sprite not loaded"));
}
