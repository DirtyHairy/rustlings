use super::{sprite::Bitmap, tile_set::PALETTE_SIZE};

#[derive(Clone)]
pub struct SpecialBackground {
    pub palette: [(usize, usize, usize); PALETTE_SIZE],
    pub bitmap: Bitmap,
}
