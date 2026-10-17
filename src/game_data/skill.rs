use strum::{Display, EnumCount, FromRepr, VariantArray};

#[derive(Clone, Copy, PartialEq, Default, VariantArray, EnumCount, FromRepr, Display)]
pub enum Skill {
    #[default]
    Climber = 0,
    Floater = 1,
    Bomber = 2,
    Blocker = 3,
    Builder = 4,
    Basher = 5,
    Miner = 6,
    Digger = 7,
}
