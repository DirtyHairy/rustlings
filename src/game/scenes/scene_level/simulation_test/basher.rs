use rustlings::game_data::file::main::MaskSprite;

use crate::{
    scenes::scene_level::{
        simulation::{MAX_BASH_DOWN, test::fixture::TerrainFixtureBuilder},
        terrain_diff::{TerrainDiff, TerrainDiffKind},
    },
    state::{Activity, Direction, LemmingAnimation, LemmingState, ObjectState, TerrainProps},
};

#[test]
fn basher_does_nothing_on_plain_frames() {
    for frame in (0..=0)
        .chain(5..=9)
        .chain(15..=15)
        .chain(16..=16)
        .chain(21..=25)
        .chain(31..=31)
    {
        let mut terrain_fixture = TerrainFixtureBuilder::new(20, 20)
            .with_row(0, 10, 20, TerrainProps::new())
            .build();

        let mut objects_fixture: Vec<ObjectState> = Vec::new();

        let lemming_fixture = LemmingState {
            frame,
            ..LemmingState::fixture(5, 10, crate::state::Direction::Right, Activity::Bashing)
        };
        let mut lemming = lemming_fixture.clone();

        lemming.tick(&mut terrain_fixture, &mut objects_fixture);

        assert_eq!(
            lemming,
            LemmingState {
                frame: (frame + 1) % 32,
                ..lemming_fixture
            }
        );
    }
}

#[test]
fn basher_bashes_on_bash_frames() {
    for frame in (1..=4).chain(17..=20) {
        let mut terrain_fixture = TerrainFixtureBuilder::new(20, 20)
            .with_row(0, 10, 20, TerrainProps::new())
            .with_block(10, 0, 10, 10, TerrainProps::new().with_solid(true))
            .build();

        let mut objects_fixture: Vec<ObjectState> = Vec::new();

        let lemming_fixture = LemmingState {
            frame,
            ..LemmingState::fixture(5, 10, crate::state::Direction::Right, Activity::Bashing)
        };
        let mut lemming = lemming_fixture.clone();

        lemming.tick(&mut terrain_fixture, &mut objects_fixture);

        assert_eq!(
            lemming,
            LemmingState {
                frame: frame + 1,
                ignore_blocker: true,
                ..lemming_fixture
            }
        );

        let (foot_x, foot_y) = LemmingAnimation::Bashing.foot();

        assert_eq!(
            terrain_fixture.sorted_diff(),
            &[TerrainDiff {
                kind: TerrainDiffKind::Mask(MaskSprite::BashR, (frame % 16) - 1),
                x: 5 - foot_x as i32,
                y: 10 - foot_y as i32
            }]
        );
    }
}

#[test]
fn basher_mask_orientation_depends_on_direction() {
    let mut terrain_fixture = TerrainFixtureBuilder::new(20, 20)
        .with_row(0, 10, 20, TerrainProps::new())
        .build();

    let mut objects_fixture: Vec<ObjectState> = Vec::new();

    let lemming_fixture = LemmingState {
        frame: 1,
        ..LemmingState::fixture(5, 10, crate::state::Direction::Left, Activity::Bashing)
    };
    let mut lemming = lemming_fixture.clone();

    lemming.tick(&mut terrain_fixture, &mut objects_fixture);

    let (foot_x, foot_y) = LemmingAnimation::Bashing.foot();

    assert_eq!(
        terrain_fixture.sorted_diff(),
        &[TerrainDiff {
            kind: TerrainDiffKind::Mask(MaskSprite::BashL, 0),
            x: 5 - foot_x as i32,
            y: 10 - foot_y as i32
        }]
    );
}

#[test]
fn basher_converts_to_walker_without_block_on_frame_4() {
    let mut terrain_fixture = TerrainFixtureBuilder::new(20, 20)
        .with_row(0, 10, 20, TerrainProps::new())
        .build();

    let mut objects_fixture: Vec<ObjectState> = Vec::new();

    let lemming_fixture = LemmingState {
        frame: 4,
        ..LemmingState::fixture(5, 10, crate::state::Direction::Right, Activity::Bashing)
    };
    let mut lemming = lemming_fixture.clone();

    lemming.tick(&mut terrain_fixture, &mut objects_fixture);

    assert_eq!(
        lemming,
        LemmingState {
            frame: 0,
            activity: Activity::Walking,
            animation: Some(LemmingAnimation::Walking),
            ignore_blocker: true,
            ..lemming_fixture
        }
    );
}

#[test]
fn basher_does_not_convert_to_walker_without_block_on_frame_20() {
    let mut terrain_fixture = TerrainFixtureBuilder::new(20, 20)
        .with_row(0, 10, 20, TerrainProps::new())
        .build();

    let mut objects_fixture: Vec<ObjectState> = Vec::new();

    let lemming_fixture = LemmingState {
        frame: 20,
        ..LemmingState::fixture(5, 10, crate::state::Direction::Right, Activity::Bashing)
    };
    let mut lemming = lemming_fixture.clone();

    lemming.tick(&mut terrain_fixture, &mut objects_fixture);

    assert_eq!(
        lemming,
        LemmingState {
            frame: 21,
            ignore_blocker: true,
            ..lemming_fixture
        }
    );
}

#[test]
fn basher_moves_on_move_frames() {
    for frame in (10..=14).chain(26..=30) {
        let mut terrain_fixture = TerrainFixtureBuilder::new(20, 20)
            .with_row(0, 10, 20, TerrainProps::new())
            .build();

        let mut objects_fixture: Vec<ObjectState> = Vec::new();

        let lemming_fixture = LemmingState {
            frame,
            ..LemmingState::fixture(5, 10, crate::state::Direction::Right, Activity::Bashing)
        };
        let mut lemming = lemming_fixture.clone();

        lemming.tick(&mut terrain_fixture, &mut objects_fixture);

        assert_eq!(
            lemming,
            LemmingState {
                frame: frame + 1,
                x: 6,
                ..lemming_fixture
            }
        );
    }
}

#[test]
fn basher_steps_down_up_to_2_pixels() {
    for step in 1..=MAX_BASH_DOWN as i32 {
        let mut terrain_fixture = TerrainFixtureBuilder::new(20, 20)
            .with_row(0, 10, 20, TerrainProps::new())
            .build();

        let mut objects_fixture: Vec<ObjectState> = Vec::new();

        let lemming_fixture = LemmingState {
            frame: 10,
            ..LemmingState::fixture(
                5,
                10 - step,
                crate::state::Direction::Right,
                Activity::Bashing,
            )
        };
        let mut lemming = lemming_fixture.clone();

        lemming.tick(&mut terrain_fixture, &mut objects_fixture);

        assert_eq!(
            lemming,
            LemmingState {
                frame: 11,
                x: 6,
                y: 10,
                ..lemming_fixture
            }
        );
    }
}

#[test]
fn basher_falls_after_more_than_2_pixels() {
    let mut terrain_fixture = TerrainFixtureBuilder::new(20, 20)
        .with_row(0, 10, 20, TerrainProps::new())
        .build();

    let mut objects_fixture: Vec<ObjectState> = Vec::new();

    let lemming_fixture = LemmingState {
        frame: 10,
        ..LemmingState::fixture(5, 7, crate::state::Direction::Right, Activity::Bashing)
    };
    let mut lemming = lemming_fixture.clone();

    lemming.tick(&mut terrain_fixture, &mut objects_fixture);

    assert_eq!(
        lemming,
        LemmingState {
            frame: 0,
            x: 6,
            y: 10,
            activity: Activity::Falling(Default::default()),
            animation: Some(LemmingAnimation::Falling),
            ..lemming_fixture
        }
    );
}

#[test]
fn basher_turns_around_and_walks_at_steel() {
    let mut terrain_fixture = TerrainFixtureBuilder::new(20, 20)
        .with_row(0, 10, 20, TerrainProps::new())
        .with(14, 2, TerrainProps::new().with_steel(true))
        .build();

    let mut objects_fixture: Vec<ObjectState> = Vec::new();

    let lemming_fixture = LemmingState {
        frame: 10,
        ..LemmingState::fixture(5, 10, crate::state::Direction::Right, Activity::Bashing)
    };
    let mut lemming = lemming_fixture.clone();

    lemming.tick(&mut terrain_fixture, &mut objects_fixture);

    assert_eq!(
        lemming,
        LemmingState {
            frame: 0,
            x: 6,
            y: 10,
            direction: Direction::Left,
            activity: Activity::Walking,
            animation: Some(LemmingAnimation::Walking),
            ..lemming_fixture
        }
    );
}

#[test]
fn right_facing_basher_turns_around_and_walks_at_one_way_left() {
    let mut terrain_fixture = TerrainFixtureBuilder::new(20, 20)
        .with_row(0, 10, 20, TerrainProps::new())
        .with(14, 2, TerrainProps::new().with_one_way_left(true))
        .build();

    let mut objects_fixture: Vec<ObjectState> = Vec::new();

    let lemming_fixture = LemmingState {
        frame: 10,
        ..LemmingState::fixture(5, 10, crate::state::Direction::Right, Activity::Bashing)
    };
    let mut lemming = lemming_fixture.clone();

    lemming.tick(&mut terrain_fixture, &mut objects_fixture);

    assert_eq!(
        lemming,
        LemmingState {
            frame: 0,
            x: 6,
            y: 10,
            direction: Direction::Left,
            activity: Activity::Walking,
            animation: Some(LemmingAnimation::Walking),
            ..lemming_fixture
        }
    );
}

#[test]
fn right_facing_basher_does_not_turn_around_and_walk_at_one_way_right() {
    let mut terrain_fixture = TerrainFixtureBuilder::new(20, 20)
        .with_row(0, 10, 20, TerrainProps::new())
        .with(14, 2, TerrainProps::new().with_one_way_right(true))
        .build();

    let mut objects_fixture: Vec<ObjectState> = Vec::new();

    let lemming_fixture = LemmingState {
        frame: 10,
        ..LemmingState::fixture(5, 10, crate::state::Direction::Right, Activity::Bashing)
    };
    let mut lemming = lemming_fixture.clone();

    lemming.tick(&mut terrain_fixture, &mut objects_fixture);

    assert_eq!(
        lemming,
        LemmingState {
            frame: 11,
            x: 6,
            y: 10,
            ..lemming_fixture
        }
    );
}

#[test]
fn left_facing_basher_turns_around_and_walks_at_one_way_right() {
    let mut terrain_fixture = TerrainFixtureBuilder::new(20, 20)
        .with_row(0, 10, 20, TerrainProps::new())
        .with(1, 2, TerrainProps::new().with_one_way_right(true))
        .build();

    let mut objects_fixture: Vec<ObjectState> = Vec::new();

    let lemming_fixture = LemmingState {
        frame: 10,
        ..LemmingState::fixture(10, 10, crate::state::Direction::Left, Activity::Bashing)
    };
    let mut lemming = lemming_fixture.clone();

    lemming.tick(&mut terrain_fixture, &mut objects_fixture);

    assert_eq!(
        lemming,
        LemmingState {
            frame: 0,
            x: 9,
            y: 10,
            direction: Direction::Right,
            activity: Activity::Walking,
            animation: Some(LemmingAnimation::Walking),
            ..lemming_fixture
        }
    );
}

#[test]
fn left_facing_basher_does_not_turn_around_and_walk_at_one_way_left() {
    let mut terrain_fixture = TerrainFixtureBuilder::new(20, 20)
        .with_row(0, 10, 20, TerrainProps::new())
        .with(1, 2, TerrainProps::new().with_one_way_left(true))
        .build();

    let mut objects_fixture: Vec<ObjectState> = Vec::new();

    let lemming_fixture = LemmingState {
        frame: 10,
        ..LemmingState::fixture(10, 10, crate::state::Direction::Left, Activity::Bashing)
    };
    let mut lemming = lemming_fixture.clone();

    lemming.tick(&mut terrain_fixture, &mut objects_fixture);

    assert_eq!(
        lemming,
        LemmingState {
            frame: 11,
            x: 9,
            y: 10,
            ..lemming_fixture
        }
    );
}
