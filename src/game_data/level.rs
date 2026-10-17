use strum::Display;

pub const NUM_LEVELS: usize = 120;
pub const LEVELS_PER_DIFFICULTY: usize = 30;

#[derive(Clone, Copy, Display)]
pub enum DifficultyRating {
    Fun,
    Tricky,
    Taxing,
    Mayhem,
}

impl From<usize> for DifficultyRating {
    fn from(value: usize) -> Self {
        match value % 4 {
            0 => Self::Fun,
            1 => Self::Tricky,
            2 => Self::Taxing,
            3 => Self::Mayhem,
            _ => unreachable!(),
        }
    }
}

pub fn decode_level_index(index: usize) -> (DifficultyRating, usize) {
    (
        DifficultyRating::from(index / LEVELS_PER_DIFFICULTY),
        (index % LEVELS_PER_DIFFICULTY) + 1,
    )
}
