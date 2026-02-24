use std::{fs, path::Path};

use anyhow::{Result, anyhow, bail};

use crate::game_data::file::encoding;
use crate::game_data::file::palette::{
    LOWER_PALETTE_FIXED, PALETTE_SIZE, PaletteEntry, expand_rgb6_to8, read_palette_entry,
};
use crate::game_data::file::read::read_byte;
use crate::game_data::file::sprite::{Bitmap, TransparencyEncoding};

const SECTION_SIZE: usize = 14400;
const VGASPEC_BITMAP_WIDTH: usize = 960;
const VGASPEC_BITMAP_HEIGHT: usize = 160;
const PALETTE_MAP: [u8; 8] = [0, 9, 10, 11, 12, 13, 14, 15];
const MAGIC_PALETTE_ENTRY: PaletteEntry = expand_rgb6_to8(0x1f, 0x1f, 0);

pub struct Content {
    pub palette: [PaletteEntry; PALETTE_SIZE],
    pub bitmap: Bitmap,
}

pub fn read_vgaspec(path: &Path, index: usize) -> Result<Content> {
    let filename = format!("vgaspec{}.dat", index);
    println!("reading {}", &filename);

    let data = fs::read(path.join(&filename).as_os_str())?;

    read_compressed_data(&data)
}

fn read8(data: &[u8], offset: usize) -> Result<u8> {
    Ok(read_byte(data, offset)?.0)
}

fn read_bitmap_section(src: &[u8], index: usize, dest: &mut [u8]) -> Result<usize> {
    let mut i_source = index;
    let mut i_dest = 0;

    loop {
        let block_tag = read8(src, i_source)?;
        i_source += 1;

        if block_tag <= 0x7f {
            let len = block_tag as usize + 1;

            dest.get_mut(i_dest..i_dest + len)
                .ok_or(anyhow!("read vgaspec section: section overflow"))?
                .copy_from_slice(
                    src.get(i_source..i_source + len)
                        .ok_or(anyhow!("read vgaspec section: source overflow"))?,
                );

            i_source += len;
            i_dest += len;
        } else if block_tag == 0x80 {
            break;
        } else {
            let byte = read8(src, i_source)?;
            let len = 128 - (block_tag as usize - 0x81);
            i_source += 1;

            dest.get_mut(i_dest..i_dest + len)
                .ok_or(anyhow!("read vgaspec section: section overflow"))?
                .fill(byte);

            i_dest += len;
        }
    }

    if i_dest == SECTION_SIZE {
        Ok(i_source)
    } else {
        bail!("invalid vgaspec section");
    }
}

fn read_compressed_data(compressed_data: &[u8]) -> Result<Content> {
    let decompressed_sections = encoding::datfile::parse(compressed_data)?;
    if decompressed_sections.sections.len() != 1 {
        bail!("vgaspec: too man sections");
    }

    let data = &decompressed_sections.sections[0].data;

    let mut palette: [PaletteEntry; PALETTE_SIZE] = [(0, 0, 0); 16];

    for i in 0..8 {
        palette[8 + i] = read_palette_entry(data, 3 * i)?.0;
        if i != 7 {
            palette[i] = LOWER_PALETTE_FIXED[i];
        };
    }

    palette[8] = MAGIC_PALETTE_ENTRY;
    palette[7] = palette[8];

    let mut i_source = 40;
    let mut bitmap_data: Vec<u8> = vec![0; SECTION_SIZE];
    let mut section_bitmaps: Vec<Bitmap> = Vec::with_capacity(4);

    for _i in 0..4 {
        i_source = read_bitmap_section(data, i_source, &mut bitmap_data)?;

        section_bitmaps.push(Bitmap::read_planar(
            VGASPEC_BITMAP_WIDTH,
            VGASPEC_BITMAP_HEIGHT / 4,
            3,
            &bitmap_data,
            TransparencyEncoding::Black,
        )?)
    }

    let mut bitmap: Vec<u8> = vec![0; VGASPEC_BITMAP_HEIGHT * VGASPEC_BITMAP_WIDTH];
    let mut transparency: Vec<bool> = vec![false; VGASPEC_BITMAP_HEIGHT * VGASPEC_BITMAP_WIDTH];

    for i in 0..4 {
        let chunk_size = VGASPEC_BITMAP_HEIGHT * VGASPEC_BITMAP_WIDTH / 4;
        bitmap[i * chunk_size..(i + 1) * chunk_size].copy_from_slice(&section_bitmaps[i].data);
        transparency[i * chunk_size..(i + 1) * chunk_size]
            .copy_from_slice(&section_bitmaps[i].transparency);
    }

    let mut i: usize = 0;
    for _ in 0..VGASPEC_BITMAP_HEIGHT {
        for _ in 0..VGASPEC_BITMAP_WIDTH {
            bitmap[i] = PALETTE_MAP[bitmap[i] as usize];
            i += 1;
        }
    }

    let bitmap = Bitmap {
        width: VGASPEC_BITMAP_WIDTH,
        height: VGASPEC_BITMAP_HEIGHT,
        data: bitmap,
        transparency,
    };

    Ok(Content { palette, bitmap })
}
