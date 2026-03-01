pub const NUM_LEVELS: usize = 120;
pub const LEVELS_PER_DIFFICULTY: usize = 30;

#[derive(Clone, Copy)]
pub enum DifficultyRating {
    Fun,
    Tricky,
    Taxing,
    Mayhem,
}

impl ToString for DifficultyRating {
    fn to_string(&self) -> String {
        match self {
            Self::Fun => "Fun".to_string(),
            Self::Tricky => "Tricky".to_string(),
            Self::Taxing => "Taxing".to_string(),
            Self::Mayhem => "Mayhem".to_string(),
        }
    }
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
        DifficultyRating::from(index),
        (index % LEVELS_PER_DIFFICULTY) + 1,
    )
}
