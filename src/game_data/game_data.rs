use super::{Level, SpecialBackground, Sprite, TileSet, NUM_LEMMING_SPRITES, PALETTE_SIZE};

#[derive(Clone)]
pub struct GameData {
    pub levels: Vec<Level>,
    pub tilesets: Vec<TileSet>,
    pub special_backgrounds: Vec<SpecialBackground>,
    pub lemming_sprites: [Sprite; NUM_LEMMING_SPRITES],
    pub static_palette: [(usize, usize, usize); PALETTE_SIZE],
}
