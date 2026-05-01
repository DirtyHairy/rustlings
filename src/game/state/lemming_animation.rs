use rustlings::game_data::file::main::{LEMMING_SPRITE_LAYOUT, LemmingSprite};
use strum::{EnumCount, FromRepr, VariantArray};

use crate::state::Direction;

#[derive(Clone, Copy, PartialEq, Default, EnumCount, VariantArray, FromRepr)]
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
    const FOOT: [(usize, usize); LemmingAnimation::COUNT] = {
        let mut foot = [(0, 0); LemmingAnimation::COUNT];

        let mut i = 0;
        while i < LemmingAnimation::COUNT {
            let animation = LemmingAnimation::from_repr(i).unwrap();
            foot[i] = match animation {
                Self::Digging => (8, 12),
                Self::Explosion => (16, 25),
                _ => (
                    LEMMING_SPRITE_LAYOUT[animation.sprite() as usize].1 / 2,
                    LEMMING_SPRITE_LAYOUT[animation.sprite() as usize].2,
                ),
            };

            i += 1;
        }

        foot
    };

    pub fn foot(self) -> (usize, usize) {
        Self::FOOT[self as usize]
    }

    pub const fn frame_count(self) -> usize {
        LEMMING_SPRITE_LAYOUT[self as usize].0
    }

    pub fn mirror(self, direction: Direction) -> bool {
        direction == Direction::Left && (self as usize) < (Self::Exitting as usize)
    }

    pub const fn sprite(self) -> LemmingSprite {
        match self {
            Self::Walking => LemmingSprite::WalkingR,
            Self::Jumping => LemmingSprite::JumpingR,
            Self::Climbing => LemmingSprite::ClimbingR,
            Self::Hoisting => LemmingSprite::HoistingR,
            Self::Building => LemmingSprite::BuildingR,
            Self::Bashing => LemmingSprite::BashingR,
            Self::Mining => LemmingSprite::MiningR,
            Self::Falling => LemmingSprite::FallingR,
            Self::PreUmbrella => LemmingSprite::PreUmbrellaR,
            Self::Umbrella => LemmingSprite::UmbrellaR,
            Self::Exitting => LemmingSprite::Exitting,
            Self::Shrugging => LemmingSprite::ShruggingR,
            Self::Frying => LemmingSprite::Frying,
            Self::Blocking => LemmingSprite::Blocking,
            Self::OhNo => LemmingSprite::OhNo,
            Self::Explosion => LemmingSprite::Explosion,
            Self::Digging => LemmingSprite::Digging,
            Self::Drowning => LemmingSprite::Drowning,
            Self::Splatting => LemmingSprite::Splatting,
        }
    }
}
