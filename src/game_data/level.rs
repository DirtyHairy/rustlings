use strum::{Display, EnumCount, FromRepr};

pub const NUM_LEVELS: usize = 120;
pub const LEVELS_PER_DIFFICULTY: usize = 30;

#[derive(Clone, Copy, Display, Default, FromRepr, EnumCount)]
pub enum DifficultyRating {
    #[default]
    Fun,
    Tricky,
    Taxing,
    Mayhem,
}

impl DifficultyRating {
    pub fn next(self) -> Self {
        Self::from_repr((self as usize + 1) % DifficultyRating::COUNT).expect("unreachable")
    }

    pub fn previous(self) -> Self {
        Self::from_repr((self as usize + DifficultyRating::COUNT - 1) % DifficultyRating::COUNT)
            .expect("unreachable")
    }
}

pub fn decode_level_index(index: usize) -> (DifficultyRating, usize) {
    (
        DifficultyRating::from_repr((index % NUM_LEVELS) / LEVELS_PER_DIFFICULTY).unwrap(),
        (index % LEVELS_PER_DIFFICULTY) + 1,
    )
}
