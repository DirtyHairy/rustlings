use rustlings::game_data::file::main::MaskSprite;

use crate::{
    scenes::scene_level::{
        simulation::test::fixture::TerrainFixtureBuilder,
        terrain_diff::{TerrainDiff, TerrainDiffKind},
    },
    state::{Activity, Direction, LemmingAnimation, LemmingState, ObjectState, TerrainProps},
};

#[test]
fn miner_facing_right_mines_right_at_frames_0_1() {
    for frame in [0, 1] {
        let mut terrain_fixture = TerrainFixtureBuilder::new(20, 20)
            .with_block(0, 10, 20, 5, TerrainProps::new())
            .build();

        let mut objects_fixture: Vec<ObjectState> = Vec::new();

        let lemming_fixture = LemmingState {
            frame,
            ..LemmingState::fixture(5, 10, Direction::Right, Activity::Mining)
        };
        let mut lemming = lemming_fixture.clone();

        lemming.tick(&mut terrain_fixture, &mut objects_fixture);

        let (foot_x, foot_y) = LemmingAnimation::Mining.foot();

        assert_eq!(
            terrain_fixture.sorted_diff(),
            &[TerrainDiff {
                kind: TerrainDiffKind::Mask(MaskSprite::MineR, frame),
                x: 5 - foot_x as i32 + frame as i32,
                y: 11 - foot_y as i32 + frame as i32
            }]
        );

        assert_eq!(
            lemming,
            LemmingState {
                frame: frame + 1,
                y: 11,
                ignore_blocker: true,
                ..lemming_fixture
            }
        )
    }
}

#[test]
fn miner_facing_left_mines_left_at_frames_0_1() {
    for frame in [0, 1] {
        let mut terrain_fixture = TerrainFixtureBuilder::new(20, 20)
            .with_block(0, 10, 20, 5, TerrainProps::new())
            .build();

        let mut objects_fixture: Vec<ObjectState> = Vec::new();

        let lemming_fixture = LemmingState {
            frame,
            ..LemmingState::fixture(5, 10, Direction::Left, Activity::Mining)
        };
        let mut lemming = lemming_fixture.clone();

        lemming.tick(&mut terrain_fixture, &mut objects_fixture);

        let (foot_x, foot_y) = LemmingAnimation::Mining.foot();

        assert_eq!(
            terrain_fixture.sorted_diff(),
            &[TerrainDiff {
                kind: TerrainDiffKind::Mask(MaskSprite::MineL, frame),
                x: 5 - foot_x as i32 - frame as i32,
                y: 11 - foot_y as i32 + frame as i32
            }]
        );

        assert_eq!(
            lemming,
            LemmingState {
                frame: frame + 1,
                y: 11,
                ignore_blocker: true,
                ..lemming_fixture
            }
        )
    }
}

#[test]
fn miner_walks_two_steps_in_frames_2_and_14() {
    for frame in [2, 14] {
        for direction in [Direction::Left, Direction::Right] {
            let mut terrain_fixture = TerrainFixtureBuilder::new(20, 20)
                .with_block(0, 10, 20, 5, TerrainProps::new())
                .build();

            let mut objects_fixture: Vec<ObjectState> = Vec::new();

            let lemming_fixture = LemmingState {
                frame,
                ..LemmingState::fixture(5, 10, direction, Activity::Mining)
            };
            let mut lemming = lemming_fixture.clone();

            lemming.tick(&mut terrain_fixture, &mut objects_fixture);

            assert_eq!(
                lemming,
                LemmingState {
                    frame: frame + 1,
                    y: lemming.y,
                    x: 5 + direction.delta(2),
                    ..lemming_fixture
                }
            );
        }
    }
}

#[test]
fn miner_descends_in_frame_2() {
    let mut terrain_fixture = TerrainFixtureBuilder::new(20, 20)
        .with_block(0, 10, 20, 5, TerrainProps::new())
        .build();

    let mut objects_fixture: Vec<ObjectState> = Vec::new();

    let lemming_fixture = LemmingState {
        frame: 2,
        ..LemmingState::fixture(5, 10, Direction::Right, Activity::Mining)
    };
    let mut lemming = lemming_fixture.clone();

    lemming.tick(&mut terrain_fixture, &mut objects_fixture);

    assert_eq!(
        lemming,
        LemmingState {
            frame: 3,
            y: 12,
            x: 7,
            ..lemming_fixture
        }
    );
}

#[test]
fn miner_turns_and_walks_on_right_boundary_in_frame_2_and_14() {
    for frame in [2, 14] {
        for x in [17, 18] {
            let mut terrain_fixture = TerrainFixtureBuilder::new(20, 20)
                .with_block(0, 10, 20, 5, TerrainProps::new())
                .build();

            let mut objects_fixture: Vec<ObjectState> = Vec::new();

            let lemming_fixture = LemmingState {
                frame,
                ..LemmingState::fixture(x, 10, Direction::Right, Activity::Mining)
            };
            let mut lemming = lemming_fixture.clone();

            lemming.tick(&mut terrain_fixture, &mut objects_fixture);

            assert_eq!(
                lemming,
                LemmingState {
                    frame: 0,
                    activity: Activity::Walking,
                    animation: Some(LemmingAnimation::Walking),
                    direction: Direction::Left,
                    y: 11,
                    x: 19,
                    ..lemming_fixture
                }
            );
        }
    }
}

#[test]
fn miner_turns_and_walks_on_left_boundary_in_frame_2_and_14() {
    for frame in [2, 14] {
        for x in [1, 2] {
            let mut terrain_fixture = TerrainFixtureBuilder::new(20, 20)
                .with_block(0, 10, 20, 5, TerrainProps::new())
                .build();

            let mut objects_fixture: Vec<ObjectState> = Vec::new();

            let lemming_fixture = LemmingState {
                frame,
                ..LemmingState::fixture(x, 10, Direction::Left, Activity::Mining)
            };
            let mut lemming = lemming_fixture.clone();

            lemming.tick(&mut terrain_fixture, &mut objects_fixture);

            assert_eq!(
                lemming,
                LemmingState {
                    frame: 0,
                    activity: Activity::Walking,
                    animation: Some(LemmingAnimation::Walking),
                    direction: Direction::Right,
                    y: 11,
                    x: 0,
                    ..lemming_fixture
                }
            );
        }
    }
}

#[test]
fn miner_falls_in_frame_2_without_support() {
    let mut terrain_fixture = TerrainFixtureBuilder::new(20, 20)
        .with_block(0, 10, 20, 2, TerrainProps::new())
        .build();

    let mut objects_fixture: Vec<ObjectState> = Vec::new();

    let lemming_fixture = LemmingState {
        frame: 2,
        ..LemmingState::fixture(5, 10, Direction::Right, Activity::Mining)
    };
    let mut lemming = lemming_fixture.clone();

    lemming.tick(&mut terrain_fixture, &mut objects_fixture);

    assert_eq!(
        lemming,
        LemmingState {
            frame: 0,
            activity: Activity::Falling(Default::default()),
            animation: Some(LemmingAnimation::Falling),
            y: 12,
            x: 7,
            ..lemming_fixture
        }
    );
}

#[test]
fn miner_falls_in_frame_14_without_support() {
    let mut terrain_fixture = TerrainFixtureBuilder::new(20, 20)
        .with_row(0, 10, 20, TerrainProps::new())
        .build();

    let mut objects_fixture: Vec<ObjectState> = Vec::new();

    let lemming_fixture = LemmingState {
        frame: 14,
        ..LemmingState::fixture(5, 10, Direction::Right, Activity::Mining)
    };
    let mut lemming = lemming_fixture.clone();

    lemming.tick(&mut terrain_fixture, &mut objects_fixture);

    assert_eq!(
        lemming,
        LemmingState {
            frame: 0,
            activity: Activity::Falling(Default::default()),
            animation: Some(LemmingAnimation::Falling),
            y: 11,
            x: 7,
            ..lemming_fixture
        }
    );
}

#[test]
fn miner_turns_and_walks_over_steel_in_frame_2_and_14() {
    for frame in [2, 14] {
        let y_target: u32 = if frame == 2 { 12 } else { 11 };

        let mut terrain_fixture = TerrainFixtureBuilder::new(20, 20)
            .with_block(0, 10, 20, 3, TerrainProps::new())
            .with(7, y_target, TerrainProps::new().with_steel(true))
            .build();

        let mut objects_fixture: Vec<ObjectState> = Vec::new();

        let lemming_fixture = LemmingState {
            frame,
            ..LemmingState::fixture(5, 10, Direction::Right, Activity::Mining)
        };
        let mut lemming = lemming_fixture.clone();

        lemming.tick(&mut terrain_fixture, &mut objects_fixture);

        assert_eq!(
            lemming,
            LemmingState {
                frame: 0,
                activity: Activity::Walking,
                animation: Some(LemmingAnimation::Walking),
                direction: Direction::Left,
                y: y_target as i32,
                x: 7,
                ..lemming_fixture
            }
        );
    }
}

#[test]
fn miner_right_turns_and_walks_over_one_way_right_2_and_14() {
    for frame in [2, 14] {
        let y_target: u32 = if frame == 2 { 12 } else { 11 };

        let mut terrain_fixture = TerrainFixtureBuilder::new(20, 20)
            .with_block(0, 10, 20, 3, TerrainProps::new())
            .with(7, y_target, TerrainProps::new().with_one_way_right(true))
            .build();

        let mut objects_fixture: Vec<ObjectState> = Vec::new();

        let lemming_fixture = LemmingState {
            frame,
            ..LemmingState::fixture(5, 10, Direction::Right, Activity::Mining)
        };
        let mut lemming = lemming_fixture.clone();

        lemming.tick(&mut terrain_fixture, &mut objects_fixture);

        assert_eq!(
            lemming,
            LemmingState {
                frame: 0,
                activity: Activity::Walking,
                animation: Some(LemmingAnimation::Walking),
                direction: Direction::Left,
                y: y_target as i32,
                x: 7,
                ..lemming_fixture
            }
        );
    }
}

#[test]
fn miner_right_turns_and_walks_over_one_way_left_2_and_14() {
    for frame in [2, 14] {
        let y_target: u32 = if frame == 2 { 12 } else { 11 };

        let mut terrain_fixture = TerrainFixtureBuilder::new(20, 20)
            .with_block(0, 10, 20, 3, TerrainProps::new())
            .with(7, y_target, TerrainProps::new().with_one_way_left(true))
            .build();

        let mut objects_fixture: Vec<ObjectState> = Vec::new();

        let lemming_fixture = LemmingState {
            frame,
            ..LemmingState::fixture(5, 10, Direction::Right, Activity::Mining)
        };
        let mut lemming = lemming_fixture.clone();

        lemming.tick(&mut terrain_fixture, &mut objects_fixture);

        assert_eq!(
            lemming,
            LemmingState {
                frame: 0,
                activity: Activity::Walking,
                animation: Some(LemmingAnimation::Walking),
                direction: Direction::Left,
                y: y_target as i32,
                x: 7,
                ..lemming_fixture
            }
        );
    }
}

#[test]
fn miner_left_turns_and_walks_over_one_way_right_2_and_14() {
    for frame in [2, 14] {
        let y_target: u32 = if frame == 2 { 12 } else { 11 };

        let mut terrain_fixture = TerrainFixtureBuilder::new(20, 20)
            .with_block(0, 10, 20, 3, TerrainProps::new())
            .with(3, y_target, TerrainProps::new().with_one_way_right(true))
            .build();

        let mut objects_fixture: Vec<ObjectState> = Vec::new();

        let lemming_fixture = LemmingState {
            frame,
            ..LemmingState::fixture(5, 10, Direction::Left, Activity::Mining)
        };
        let mut lemming = lemming_fixture.clone();

        lemming.tick(&mut terrain_fixture, &mut objects_fixture);

        assert_eq!(
            lemming,
            LemmingState {
                frame: 0,
                activity: Activity::Walking,
                animation: Some(LemmingAnimation::Walking),
                direction: Direction::Right,
                y: y_target as i32,
                x: 3,
                ..lemming_fixture
            }
        );
    }
}

#[test]
fn miner_right_left_does_not_turn_and_walks_over_one_way_left_2_and_14() {
    for frame in [2, 14] {
        let y_target: u32 = if frame == 2 { 12 } else { 11 };

        let mut terrain_fixture = TerrainFixtureBuilder::new(20, 20)
            .with_block(0, 10, 20, 3, TerrainProps::new())
            .with(3, y_target, TerrainProps::new().with_one_way_left(true))
            .build();

        let mut objects_fixture: Vec<ObjectState> = Vec::new();

        let lemming_fixture = LemmingState {
            frame,
            ..LemmingState::fixture(5, 10, Direction::Left, Activity::Mining)
        };
        let mut lemming = lemming_fixture.clone();

        lemming.tick(&mut terrain_fixture, &mut objects_fixture);

        assert_eq!(
            lemming,
            LemmingState {
                frame: frame + 1,
                y: y_target as i32,
                x: 3,
                ..lemming_fixture
            }
        );
    }
}

#[test]
fn miner_descends_at_frame_23() {
    let mut terrain_fixture = TerrainFixtureBuilder::new(20, 20)
        .with_block(0, 10, 20, 5, TerrainProps::new())
        .build();

    let mut objects_fixture: Vec<ObjectState> = Vec::new();

    let lemming_fixture = LemmingState {
        frame: 23,
        ..LemmingState::fixture(5, 10, Direction::Right, Activity::Mining)
    };
    let mut lemming = lemming_fixture.clone();

    lemming.tick(&mut terrain_fixture, &mut objects_fixture);

    assert_eq!(
        lemming,
        LemmingState {
            frame: 0,
            y: 12,
            ..lemming_fixture
        }
    )
}

#[test]
fn nothing_happens_in_all_other_frames() {
    for frame in (3..=13).chain(15..22) {
        let mut terrain_fixture = TerrainFixtureBuilder::new(20, 20)
            .with_block(0, 10, 20, 5, TerrainProps::new())
            .build();

        let mut objects_fixture: Vec<ObjectState> = Vec::new();

        let lemming_fixture = LemmingState {
            frame,
            ..LemmingState::fixture(5, 10, Direction::Right, Activity::Mining)
        };
        let mut lemming = lemming_fixture.clone();

        lemming.tick(&mut terrain_fixture, &mut objects_fixture);

        assert_eq!(
            lemming,
            LemmingState {
                frame: frame + 1,
                ..lemming_fixture
            }
        )
    }
}
