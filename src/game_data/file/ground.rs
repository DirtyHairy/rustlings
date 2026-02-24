use std::fmt;
use std::{fs, path::Path};

use anyhow::*;

use crate::game_data::file::palette::{
    LOWER_PALETTE_FIXED, PALETTE_SIZE, PaletteEntry, read_palette_entry,
};
use crate::game_data::file::read::{read_byte, read_word_be};

pub const OBJECTS_PER_TILESET: usize = 16;
pub const TILES_PER_TILESET: usize = 64;

#[derive(Default, Clone)]
pub struct ObjectInfo {
    pub animation_loops: bool,
    pub animation_start: usize,
    pub animation_end: usize,
    pub width: usize,
    pub height: usize,
    pub animation_frame_size: usize,
    pub mask_offset: usize,
    pub trigger_left: usize,
    pub trigger_top: usize,
    pub trigger_width: usize,
    pub trigger_height: usize,
    pub trigger_effect: usize,
    pub frames_offset: usize,
    pub preview_frame_offset: usize,
    pub trap_sound_effect: usize,
}

#[derive(Default, Clone)]
pub struct TerrainInfo {
    pub width: usize,
    pub height: usize,
    pub image_offset: usize,
    pub mask_offset: usize,
}

#[derive(Default, Clone)]
pub struct Palettes {
    pub custom: [PaletteEntry; PALETTE_SIZE],
    pub standard: [PaletteEntry; PALETTE_SIZE],
    pub preview: [PaletteEntry; PALETTE_SIZE],
}

pub struct Content {
    pub object_info: [ObjectInfo; OBJECTS_PER_TILESET],
    pub terrain_info: [TerrainInfo; TILES_PER_TILESET],
    pub palettes: Palettes,
}

pub fn read_ground(path: &Path, index: usize) -> Result<Content> {
    let filename = format!("ground{}o.dat", index);
    println!("reading {}", &filename);

    let data = fs::read(path.join(&filename).as_os_str())?;

    let mut offset = 0;
    let mut object_info: [ObjectInfo; OBJECTS_PER_TILESET] =
        [(); OBJECTS_PER_TILESET].map(|_| ObjectInfo::default());

    for i in 0..16 {
        let (value, new_offset) = read_object_info(&data, offset)?;
        offset = new_offset;

        object_info[i] = value;
    }

    let mut terrain_info: [TerrainInfo; TILES_PER_TILESET] =
        [(); TILES_PER_TILESET].map(|_| TerrainInfo::default());

    for i in 0..64 {
        let (value, new_offset) = read_terrain_info(&data, offset)?;
        offset = new_offset;

        terrain_info[i] = value;
    }

    let (palettes, offset) = read_palettes(&data, offset)?;

    if offset != data.len() {
        bail!(
            "extra data left in {}: read {} bytes, but got {} bytes",
            &filename,
            offset,
            data.len()
        );
    }

    Ok(Content {
        object_info,
        terrain_info,
        palettes,
    })
}

fn read_object_info(buffer: &[u8], offset: usize) -> Result<(ObjectInfo, usize)> {
    let (animation_flags, offset) = read_word_be::<usize>(buffer, offset)?;
    let (animation_start, offset) = read_byte::<usize>(buffer, offset)?;
    let (animation_end, offset) = read_byte::<usize>(buffer, offset)?;
    let (width, offset) = read_byte::<usize>(buffer, offset)?;
    let (height, offset) = read_byte::<usize>(buffer, offset)?;
    let (animation_frame_size, offset) = read_word_be::<usize>(buffer, offset)?;
    let (mask_offset, offset) = read_word_be(buffer, offset)?;
    let offset = offset + 4;
    let (trigger_left, offset) = read_word_be::<usize>(buffer, offset)?;
    let (trigger_top, offset) = read_word_be::<usize>(buffer, offset)?;
    let (trigger_width, offset) = read_byte::<usize>(buffer, offset)?;
    let (trigger_height, offset) = read_byte::<usize>(buffer, offset)?;
    let (trigger_effect, offset) = read_byte::<usize>(buffer, offset)?;
    let (frames_offset, offset) = read_word_be::<usize>(buffer, offset)?;
    let (preview_frame_offset, offset) = read_word_be::<usize>(buffer, offset)?;
    let offset = offset + 2;
    let (trap_sound_effect, offset) = read_byte::<usize>(buffer, offset)?;

    Ok((
        ObjectInfo {
            animation_loops: (animation_flags & 0x01) > 0,
            animation_start,
            animation_end,
            width,
            height,
            animation_frame_size,
            mask_offset,
            trigger_left,
            trigger_top,
            trigger_width,
            trigger_height,
            trigger_effect,
            frames_offset,
            preview_frame_offset,
            trap_sound_effect,
        },
        offset,
    ))
}

fn read_terrain_info(buffer: &[u8], offset: usize) -> Result<(TerrainInfo, usize)> {
    let (width, offset) = read_byte::<usize>(buffer, offset)?;
    let (height, offset) = read_byte::<usize>(buffer, offset)?;
    let (image_offset, offset) = read_word_be::<usize>(buffer, offset)?;
    let (mask_offset, offset) = read_word_be::<usize>(buffer, offset)?;
    let offset = offset + 2;

    Ok((
        TerrainInfo {
            width,
            height,
            image_offset,
            mask_offset,
        },
        offset,
    ))
}

fn read_palette(buffer: &[u8], offset: usize) -> Result<([PaletteEntry; 16], usize)> {
    let mut palette: [PaletteEntry; PALETTE_SIZE] = [(0, 0, 0); PALETTE_SIZE];
    let mut offset = offset;

    for i in 0..7 {
        palette[i] = LOWER_PALETTE_FIXED[i];
    }

    for i in 8..16 {
        let (entry, new_offset) = read_palette_entry(buffer, offset)?;

        palette[i] = entry;
        offset = new_offset;
    }

    palette[7] = palette[8];

    Ok((palette, offset))
}

fn read_palettes(buffer: &[u8], offset: usize) -> Result<(Palettes, usize)> {
    let offset = offset + 24;
    let (custom, offset) = read_palette(buffer, offset)?;
    let (standard, offset) = read_palette(buffer, offset)?;
    let (preview, offset) = read_palette(buffer, offset)?;

    Ok((
        Palettes {
            custom,
            standard,
            preview,
        },
        offset,
    ))
}

impl fmt::Display for ObjectInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            r#"animation_loops:          {}
animation_start:          {}
animation_end:            {}
width:                    {}
height:                   {}
animation_frame_size:     {}
mask_offset:              {}
trigger_left:             {}
trigger_top:              {}
trigger_width:            {}
trigger_height:           {}
trigger_effect:           {}
frames_offset:            {}
preview_frame:            {}
trap_sound_effect:        {}"#,
            self.animation_loops,
            self.animation_start,
            self.animation_end,
            self.width,
            self.height,
            self.animation_frame_size,
            self.mask_offset,
            self.trigger_left,
            self.trigger_top,
            self.trigger_width,
            self.trigger_height,
            self.trigger_effect,
            self.frames_offset,
            self.preview_frame_offset,
            self.trap_sound_effect,
        )
    }
}

fn format_palette_entry(
    (r, g, b): PaletteEntry,
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    write!(f, "   ({} {} {})\n", r, g, b)
}

fn format_palette(
    palette: [PaletteEntry; 16],
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    for entry in palette {
        format_palette_entry(entry, f)?;
    }

    std::fmt::Result::Ok(())
}

impl fmt::Display for Palettes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "custom:\n")?;
        format_palette(self.custom, f)?;

        write!(f, "\nstandard:\n")?;
        format_palette(self.standard, f)?;

        write!(f, "\npreview:\n")?;
        format_palette(self.preview, f)
    }
}

impl fmt::Display for TerrainInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            r#"width:          {}
height:         {}
image_offset:   {}
mask_offset:    {}"#,
            self.width, self.height, self.image_offset, self.mask_offset
        )
    }
}
