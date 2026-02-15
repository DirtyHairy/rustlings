use super::read::read_byte;
use anyhow::Result;

pub const PALETTE_SIZE: usize = 16;

pub type PaletteEntry = (u8, u8, u8);

pub const fn expand_rgb6_to8(r: u8, g: u8, b: u8) -> PaletteEntry {
    return (
        (r << 2) | (r >> 4),
        (g << 2) | (g >> 4),
        (b << 2) | (b >> 4),
    );
}

pub fn read_palette_entry(buffer: &[u8], offset: usize) -> Result<(PaletteEntry, usize)> {
    let (r, offset) = read_byte(buffer, offset)?;
    let (g, offset) = read_byte(buffer, offset)?;
    let (b, offset) = read_byte(buffer, offset)?;

    Ok((expand_rgb6_to8(r, g, b), offset))
}

pub const LOWER_PALETTE_FIXED: [PaletteEntry; 7] = [
    (0, 0, 0),
    expand_rgb6_to8(0x10, 0x10, 0x38),
    expand_rgb6_to8(0x00, 0x2c, 0x00),
    expand_rgb6_to8(0x3c, 0x34, 0x34),
    expand_rgb6_to8(0x2c, 0x2c, 0x00),
    expand_rgb6_to8(0x3c, 0x08, 0x08),
    expand_rgb6_to8(0x20, 0x20, 0x20),
];
