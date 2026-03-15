pub const NUM_ASIGNABLE_SKILLS: usize = 8;

pub const ASSIGNABLE_SKILLS: [Skill; NUM_ASIGNABLE_SKILLS] = [
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
    Faller = 8,
}

impl std::string::ToString for Skill {
    fn to_string(&self) -> String {
        let str = match self {
            Skill::Climber => "Climber",
            Skill::Floater => "Floater",
            Skill::Bomber => "Bomber",
            Skill::Blocker => "Blocker",
            Skill::Builder => "Builder",
            Skill::Basher => "Basher",
            Skill::Miner => "Miner",
            Skill::Digger => "Digger",
            Skill::Faller => "Faller",
        };

        String::from(str)
    }
}
