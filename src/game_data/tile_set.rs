use std::fmt;

use super::sprite::{Bitmap, Sprite};

pub const PALETTE_SIZE: usize = 16;
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
    pub custom: [(usize, usize, usize); PALETTE_SIZE],
    pub standard: [(usize, usize, usize); PALETTE_SIZE],
    pub preview: [(usize, usize, usize); PALETTE_SIZE],
}

#[derive(Clone)]
#[allow(dead_code)]
pub struct TileSet {
    pub object_info: [ObjectInfo; OBJECTS_PER_TILESET],
    pub terrain_info: [TerrainInfo; TILES_PER_TILESET],
    pub palettes: Palettes,
    pub object_sprites: [Option<Sprite>; OBJECTS_PER_TILESET],
    pub tiles: [Option<Bitmap>; TILES_PER_TILESET],
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
    write!(f, "   ({} {} {})\n", r, g, b)
}

fn format_palette(
    palette: [(usize, usize, usize); 16],
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
