use rustlings::game_data::file::main::{LEMMING_SPRITE_LAYOUT, LemmingSprite};
use strum::{EnumCount, FromRepr, VariantArray};

use crate::state::Direction;

#[derive(Clone, Copy, PartialEq, Default, EnumCount, VariantArray, FromRepr)]
#[cfg_attr(test, derive(Debug))]
pub enum LemmingAnimation {
    #[default]
    Walking,
    Jumping,
    Climbing,
    Hoisting,
    Building,
    Bashing,
    Mining,
    Falling,
    PreUmbrella,
    Umbrella,
    Shrugging, // last assymetric animation
    Exitting,  // first symmetric animation
    Frying,
    Blocking,
    OhNo,
    Explosion,
    Digging,
    Drowning,
    Splatting,
}

impl LemmingAnimation {
    const FOOT: [(u32, u32); LemmingAnimation::COUNT] = {
        let mut foot = [(0u32, 0u32); LemmingAnimation::COUNT];

        let mut i = 0;
        while i < LemmingAnimation::COUNT {
            let animation = LemmingAnimation::from_repr(i).unwrap();
            foot[i] = match animation {
                Self::Digging => (8, 12),
                Self::Explosion => (16, 25),
                _ => (
                    LEMMING_SPRITE_LAYOUT[animation.sprite(Direction::Right) as usize].1 / 2,
                    LEMMING_SPRITE_LAYOUT[animation.sprite(Direction::Right) as usize].2,
                ),
            };

            i += 1;
        }

        foot
    };

    pub fn foot(self) -> (u32, u32) {
        Self::FOOT[self as usize]
    }

    pub const fn frame_count(self) -> usize {
        LEMMING_SPRITE_LAYOUT[self.sprite(Direction::Right) as usize].0
    }

    pub const fn sprite(self, direction: Direction) -> LemmingSprite {
        match (self, direction) {
            (Self::Walking, Direction::Right) => LemmingSprite::WalkingR,
            (Self::Walking, Direction::Left) => LemmingSprite::WalkingL,
            (Self::Jumping, Direction::Right) => LemmingSprite::JumpingR,
            (Self::Jumping, Direction::Left) => LemmingSprite::JumpingL,
            (Self::Climbing, Direction::Right) => LemmingSprite::ClimbingR,
            (Self::Climbing, Direction::Left) => LemmingSprite::ClimbingL,
            (Self::Hoisting, Direction::Right) => LemmingSprite::HoistingR,
            (Self::Hoisting, Direction::Left) => LemmingSprite::HoistingL,
            (Self::Building, Direction::Right) => LemmingSprite::BuildingR,
            (Self::Building, Direction::Left) => LemmingSprite::BuildingL,
            (Self::Bashing, Direction::Right) => LemmingSprite::BashingR,
            (Self::Bashing, Direction::Left) => LemmingSprite::BashingL,
            (Self::Mining, Direction::Right) => LemmingSprite::MiningR,
            (Self::Mining, Direction::Left) => LemmingSprite::MiningL,
            (Self::Falling, Direction::Right) => LemmingSprite::FallingR,
            (Self::Falling, Direction::Left) => LemmingSprite::FallingL,
            (Self::PreUmbrella, Direction::Right) => LemmingSprite::PreUmbrellaR,
            (Self::PreUmbrella, Direction::Left) => LemmingSprite::PreUmbrellaL,
            (Self::Umbrella, Direction::Right) => LemmingSprite::UmbrellaR,
            (Self::Umbrella, Direction::Left) => LemmingSprite::UmbrellaL,
            (Self::Shrugging, Direction::Right) => LemmingSprite::ShruggingR,
            (Self::Shrugging, Direction::Left) => LemmingSprite::ShruggingL,
            (Self::Exitting, _) => LemmingSprite::Exitting,
            (Self::Frying, _) => LemmingSprite::Frying,
            (Self::Blocking, _) => LemmingSprite::Blocking,
            (Self::OhNo, _) => LemmingSprite::OhNo,
            (Self::Explosion, _) => LemmingSprite::Explosion,
            (Self::Digging, _) => LemmingSprite::Digging,
            (Self::Drowning, _) => LemmingSprite::Drowning,
            (Self::Splatting, _) => LemmingSprite::Splatting,
        }
    }
}
