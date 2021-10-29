use anyhow::*;
use std::{cmp::Ordering, convert::TryInto, fs, path::Path};

use crate::{bitstream, datfile, definitions::LEMMING_SPRITES, sprites::Sprite};

pub struct Content {
    pub lemming_sprites: [Sprite; 30],
}

pub fn parse(path: &Path) -> Result<Content> {
    let maindata = fs::read(path.join("main.dat").as_os_str())?;

    println!("reading {} bytes from main.dat\n", maindata.len());
    let mut offset = 0;

    let mut lemming_sprites: Vec<Sprite> = Vec::new();
    let mut i_section = 0;

    loop {
        let (header, o) = datfile::Header::read(&maindata, offset)?;
        println!("found header:\n{}", header);

        let checksum = datfile::calculate_checksum(&header, &maindata, o)?;
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
                maindata
                    .get(o..o + header.compressed_data_size - 10)
                    .ok_or(anyhow!("out of bounds decompressing section"))?
                    .to_vec(),
                header.num_bits_in_first_byte,
            ),
            &mut decompressed_section,
        )?;

        if decompressed_section.len() != header.decompressed_data_size {
            bail!("sanity check failed: decompressed data does not match header");
        }

        if i_section == 0 {
            let mut offset = 0;

            for (frame_count, width, height, bpp) in LEMMING_SPRITES {
                lemming_sprites.push(Sprite::read_planar(
                    frame_count,
                    width,
                    height,
                    bpp,
                    decompressed_section
                        .get(offset..)
                        .ok_or(anyhow!("out of bounds reading lemming sprites"))?,
                    &mut offset,
                )?);
            }
        }

        offset = o + header.compressed_data_size - 10;

        println!();
        i_section += 1;

        match offset.cmp(&maindata.len()) {
            Ordering::Equal => break,
            Ordering::Greater => panic!("bad file"),
            Ordering::Less => continue,
        };
    }

    return Ok(Content {
        lemming_sprites: lemming_sprites
            .try_into()
            .map_err(|_| ())
            .expect("internal error"),
    });
}
