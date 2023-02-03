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

pub enum Skill {
    Climber,
    Floater,
    Bomber,
    Blocker,
    Builder,
    Basher,
    Miner,
    Digger,
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
        };

        String::from(str)
    }
}
