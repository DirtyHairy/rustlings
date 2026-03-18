use std::fmt::Display;

pub const NUM_SKILLS: usize = 8;

pub const SKILLS: [Skill; NUM_SKILLS] = [
    Skill::Climber,
    Skill::Floater,
    Skill::Bomber,
    Skill::Blocker,
    Skill::Builder,
    Skill::Basher,
    Skill::Miner,
    Skill::Digger,
];

#[derive(Clone, Copy, PartialEq)]
pub enum Skill {
    Climber = 0,
    Floater = 1,
    Bomber = 2,
    Blocker = 3,
    Builder = 4,
    Basher = 5,
    Miner = 6,
    Digger = 7,
}

impl Display for Skill {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Skill::Climber => "Climber",
            Skill::Floater => "Floater",
            Skill::Bomber => "Bomber",
            Skill::Blocker => "Blocker",
            Skill::Builder => "Builder",
            Skill::Basher => "Basher",
            Skill::Miner => "Miner",
            Skill::Digger => "Digger",
        };

        write!(f, "{}", str)
    }
}
