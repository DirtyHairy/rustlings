use core::fmt;
use std::{convert::TryInto, fs, path::Path};

use anyhow::*;

use super::palette::PALETTE_FIXED;

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

pub struct TerrainInfo {
    pub width: usize,
    pub height: usize,
    pub image_offset: usize,
    pub mask_offset: usize,
}

pub struct Palettes {
    pub custom: [(usize, usize, usize); 16],
    pub standard: [(usize, usize, usize); 16],
    pub preview: [(usize, usize, usize); 16],
}

pub struct Content {
    pub object_info: [ObjectInfo; 16],
    pub terrain_info: [TerrainInfo; 64],
    pub palettes: Palettes,
}

pub fn read(path: &Path, index: usize) -> Result<Content> {
    let filename = format!("ground{}o.dat", index);
    let data = fs::read(path.join(&filename).as_os_str())?;

    println!("reading {}", &filename);

    let mut offset = 0;
    let mut object_info: Vec<ObjectInfo> = Vec::new();

    for _ in 0..16 {
        let (value, new_offset) = read_object_info(&data, offset)?;
        offset = new_offset;

        object_info.push(value);
    }

    let mut terrain_info: Vec<TerrainInfo> = Vec::new();
    for _ in 0..64 {
        let (value, new_offset) = read_terrain_info(&data, offset)?;
        offset = new_offset;

        terrain_info.push(value);
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

    return Ok(Content {
        object_info: object_info
            .try_into()
            .map_err(|_| ())
            .expect("internal error"),
        terrain_info: terrain_info
            .try_into()
            .map_err(|_| ())
            .expect("internal error"),
        palettes,
    });
}

fn read_object_info(buffer: &Vec<u8>, offset: usize) -> Result<(ObjectInfo, usize)> {
    let (animation_flags, offset) = read_word(buffer, offset)?;
    let (animation_start, offset) = read_byte(buffer, offset)?;
    let (animation_end, offset) = read_byte(buffer, offset)?;
    let (width, offset) = read_byte(buffer, offset)?;
    let (height, offset) = read_byte(buffer, offset)?;
    let (animation_frame_size, offset) = read_word(buffer, offset)?;
    let (mask_offset, offset) = read_word(buffer, offset)?;
    let offset = offset + 4;
    let (trigger_left, offset) = read_word(buffer, offset)?;
    let (trigger_top, offset) = read_word(buffer, offset)?;
    let (trigger_width, offset) = read_byte(buffer, offset)?;
    let (trigger_height, offset) = read_byte(buffer, offset)?;
    let (trigger_effect, offset) = read_byte(buffer, offset)?;
    let (frames_offset, offset) = read_word(buffer, offset)?;
    let (preview_frame_offset, offset) = read_word(buffer, offset)?;
    let offset = offset + 2;
    let (trap_sound_effect, offset) = read_byte(buffer, offset)?;

    return Ok((
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
    ));
}

fn read_terrain_info(buffer: &Vec<u8>, offset: usize) -> Result<(TerrainInfo, usize)> {
    let (width, offset) = read_byte(buffer, offset)?;
    let (height, offset) = read_byte(buffer, offset)?;
    let (image_offset, offset) = read_word(buffer, offset)?;
    let (mask_offset, offset) = read_word(buffer, offset)?;
    let offset = offset + 2;

    return Ok((
        TerrainInfo {
            width,
            height,
            image_offset,
            mask_offset,
        },
        offset,
    ));
}

fn read_palette_entry(buffer: &Vec<u8>, offset: usize) -> Result<((usize, usize, usize), usize)> {
    let (r, offset) = read_byte(buffer, offset)?;
    let (g, offset) = read_byte(buffer, offset)?;
    let (b, offset) = read_byte(buffer, offset)?;

    return Ok(((r << 2, g << 2, b << 2), offset));
}
fn read_palette(buffer: &Vec<u8>, offset: usize) -> Result<([(usize, usize, usize); 16], usize)> {
    let mut palette: [(usize, usize, usize); 16] = [(0, 0, 0); 16];
    let mut offset = offset;

    for i in 0..7 {
        palette[i] = PALETTE_FIXED[i];
    }

    for i in 8..16 {
        let (entry, new_offset) = read_palette_entry(buffer, offset)?;

        palette[i] = entry;
        offset = new_offset;
    }

    palette[7] = palette[8];

    return Ok((palette, offset));
}

fn read_palettes(buffer: &Vec<u8>, offset: usize) -> Result<(Palettes, usize)> {
    let offset = offset + 24;
    let (custom, offset) = read_palette(buffer, offset)?;
    let (standard, offset) = read_palette(buffer, offset)?;
    let (preview, offset) = read_palette(buffer, offset)?;

    return Ok((
        Palettes {
            custom,
            standard,
            preview,
        },
        offset,
    ));
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
    (r, g, b): (usize, usize, usize),
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    return write!(f, "   ({} {} {})\n", r, g, b);
}

fn format_palette(
    palette: [(usize, usize, usize); 16],
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    for entry in palette {
        format_palette_entry(entry, f)?;
    }

    return std::fmt::Result::Ok(());
}

impl fmt::Display for Palettes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "custom:\n")?;
        format_palette(self.custom, f)?;

        write!(f, "\nstandard:\n")?;
        format_palette(self.standard, f)?;

        write!(f, "\npreview:\n")?;
        return format_palette(self.preview, f);
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

fn read_byte(buffer: &Vec<u8>, offset: usize) -> Result<(usize, usize)> {
    return Ok((
        *buffer
            .get(offset)
            .ok_or(anyhow!("offset {} out of bounds", offset))? as usize,
        offset + 1,
    ));
}

fn read_word(buffer: &Vec<u8>, offset: usize) -> Result<(usize, usize)> {
    return Ok((
        ((read_byte(buffer, offset)?.0 as u16) | (read_byte(buffer, offset + 1)?.0 as u16) << 8)
            as usize,
        offset + 2,
    ));
}
