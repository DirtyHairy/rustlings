use std::{fs, path::Path};

use anyhow::{Result, anyhow, bail};
use strum::EnumCount;

use crate::game_data::Bitmap;
use crate::game_data::file::encoding::datfile;
use crate::game_data::file::sprite::{Sprite, TransparencyEncoding};

pub const FONT_SKILL_PANEL_SKILLS_SIZE: usize = 11;
pub const FONT_SKILL_PANEL_SIZE: usize = 39;
pub const FONT_COUNTDOWN_SIZE: usize = 10;

const COLOR_WHITE: u8 = 0x03;
const COLOR_BLACK: u8 = 0x00;
const COLOR_DARK_GREEN: u8 = 0x02;
const COLOR_LIGHT_GREEN: u8 = 0x09;

#[derive(Copy, Clone, PartialEq, Default, EnumCount)]
pub enum LemmingSprite {
    #[default]
    WalkingR = 0,
    JumpingR = 1,
    WalkingL = 2,
    JumpingL = 3,
    Digging = 4,
    ClimbingR = 5,
    ClimbingL = 6,
    Drowning = 7,
    HoistingR = 8,
    HoistingL = 9,
    BuildingR = 10,
    BuildingL = 11,
    BashingR = 12,
    BashingL = 13,
    MiningR = 14,
    MiningL = 15,
    FallingR = 16,
    FallingL = 17,
    PreUmbrellaR = 18,
    UmbrellaR = 19,
    PreUmbrellaL = 20,
    UmbrellaL = 21,
    Splatting = 22,
    Exiting = 23,
    Frying = 24,
    Blocking = 25,
    ShruggingR = 26,
    ShruggingL = 27,
    OhNo = 28,
    Explosion = 29,
}

pub const LEMMING_SPRITE_LAYOUT: [(usize, u32, u32, usize); LemmingSprite::COUNT] = [
    (8, 16, 10, 2),
    (1, 16, 10, 2),
    (8, 16, 10, 2),
    (1, 16, 10, 2),
    (16, 16, 14, 3),
    (8, 16, 12, 2),
    (8, 16, 12, 2),
    (16, 16, 10, 2),
    (8, 16, 12, 2),
    (8, 16, 12, 2),
    (16, 16, 13, 3),
    (16, 16, 13, 3),
    (32, 16, 10, 3),
    (32, 16, 10, 3),
    (24, 16, 13, 3),
    (24, 16, 13, 3),
    (4, 16, 10, 2),
    (4, 16, 10, 2),
    (4, 16, 16, 3),
    (4, 16, 16, 3),
    (4, 16, 16, 3),
    (4, 16, 16, 3),
    (16, 16, 10, 2),
    (8, 16, 13, 2),
    (14, 16, 14, 4),
    (16, 16, 10, 2),
    (8, 16, 10, 2),
    (8, 16, 10, 2),
    (16, 16, 10, 2),
    (1, 32, 32, 3),
];

#[derive(Copy, Clone, PartialEq, Default, Debug, EnumCount)]
pub enum MaskSprite {
    #[default]
    BashR = 0,
    BashL = 1,
    MineR = 2,
    MineL = 3,
    Explosion = 4,
}

pub const MASK_SPRITE_LAYOUT: [(usize, u32, u32); MaskSprite::COUNT] = [
    (4, 16, 10),
    (4, 16, 10),
    (2, 16, 13),
    (2, 16, 13),
    (1, 16, 22),
];

pub struct Content {
    pub lemming_sprites: [Sprite; LemmingSprite::COUNT],
    pub skill_panel: Bitmap,
    pub font_skill_panel_skills: Sprite,
    pub font_skill_panel: Sprite,
    pub font_countdown: Sprite,
    pub mask_sprites: [Sprite; MaskSprite::COUNT],
}

pub fn read_main(path: &Path) -> Result<Content> {
    println!("reading main.dat");
    let maindata = fs::read(path.join("main.dat").as_os_str())?;

    let datfile::Content { sections } = datfile::parse(&maindata)?;
    if sections.len() < 3 {
        bail!("invalid main.dat");
    }

    let mut lemming_sprites: [Sprite; LemmingSprite::COUNT] = Default::default();
    let mut offset = 0;

    for (i, (frame_count, width, height, bpp)) in LEMMING_SPRITE_LAYOUT.iter().copied().enumerate()
    {
        lemming_sprites[i] = Sprite::read_planar(
            frame_count,
            width,
            height,
            bpp,
            &sections[0].data,
            &mut offset,
            (width as usize * height as usize * bpp) / 8,
            TransparencyEncoding::Black,
        )?;
    }

    let mut mask_sprites: [Sprite; MaskSprite::COUNT] = Default::default();
    offset = 0;

    for (i, (frame_count, width, height)) in MASK_SPRITE_LAYOUT.iter().copied().enumerate() {
        mask_sprites[i] = Sprite::read_planar_mapped(
            frame_count,
            width,
            height,
            1,
            &sections[1].data,
            &mut offset,
            (width * height) as usize / 8,
            TransparencyEncoding::Black,
            |c| if c == 0 { COLOR_WHITE } else { COLOR_BLACK },
        )?;
    }

    let skill_panel =
        Bitmap::read_planar(320, 40, 4, &sections[2].data, TransparencyEncoding::Black)?;

    let mut font_skill_panel_skills = Sprite::blank(4, 8, FONT_SKILL_PANEL_SKILLS_SIZE);
    let mut font_skill_panel = Sprite::blank(8, 16, FONT_SKILL_PANEL_SIZE);
    let mut font_countdown = Sprite::blank(8, 8, FONT_COUNTDOWN_SIZE);

    for i in 0..FONT_SKILL_PANEL_SKILLS_SIZE - 1 {
        let char_bitmap = Bitmap::read_planar_mapped(
            8,
            8,
            1,
            sections[2]
                .data
                .get(0x1908 + i * 0x10..)
                .ok_or(anyhow!("skill font data out of bounds"))?,
            TransparencyEncoding::Opaque,
            |x| if x == 0 { COLOR_BLACK } else { COLOR_WHITE },
        )?
        .sub(0, 0, 4, 8)?;

        font_skill_panel_skills.add_frame(&char_bitmap)?;
    }

    font_skill_panel_skills.add_frame(&Bitmap::filled(4, 8, COLOR_WHITE, false))?;

    for i in 0..FONT_SKILL_PANEL_SIZE - 1 {
        let char_bitmap = Bitmap::read_planar_mapped(
            8,
            16,
            3,
            sections[2]
                .data
                .get((0x19a0 + i * 0x30)..)
                .ok_or(anyhow!("skill panel font data out of bounds"))?,
            TransparencyEncoding::Opaque,
            |x| match x {
                0x05 => COLOR_LIGHT_GREEN,
                0x03 => COLOR_WHITE,
                0x02 => COLOR_DARK_GREEN,
                _ => 0x00,
            },
        )?;

        font_skill_panel.add_frame(&char_bitmap)?;
    }

    font_skill_panel.add_frame(&Bitmap::filled(8, 16, COLOR_BLACK, false))?;

    for i in 0..FONT_COUNTDOWN_SIZE {
        let char_bitmap = Bitmap::read_planar_mapped(
            8,
            8,
            1,
            sections[1]
                .data
                .get(0x134 + (FONT_COUNTDOWN_SIZE - 1 - i) * 8..)
                .ok_or(anyhow!("countdown font data out of bounds"))?,
            TransparencyEncoding::Black,
            |x| if x == 0 { COLOR_BLACK } else { COLOR_WHITE },
        )?;

        font_countdown.add_frame(&char_bitmap)?;
    }

    Ok(Content {
        lemming_sprites,
        skill_panel,
        font_skill_panel_skills,
        font_skill_panel,
        font_countdown,
        mask_sprites,
    })
}

pub fn resolve_skill_panel_skill_font_index(c: char) -> usize {
    match c {
        '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => {
            (c as u32 - '0' as u32) as usize
        }
        _ => 10,
    }
}

pub fn resolve_skill_panel_font_index(c: char) -> usize {
    match c {
        '%' => 0,
        '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => {
            (c as usize - '0' as usize) + 1
        }
        '-' => 11,
        'A' | 'B' | 'C' | 'D' | 'E' | 'F' | 'G' | 'H' | 'I' | 'J' | 'K' | 'L' | 'M' | 'N' | 'O'
        | 'P' | 'Q' | 'R' | 'S' | 'T' | 'U' | 'V' | 'W' | 'X' | 'Y' | 'Z' => {
            (c as usize - 'A' as usize) + 12
        }
        _ => 38,
    }
}
