use crate::{
    scenes::scene_level::simulation::test::fixture::TerrainFixtureBuilder,
    state::{
        Activity,
        Direction::{self},
        LemmingAnimation, LemmingState, ObjectsState, TerrainProps,
    },
};

#[test]
fn walker_starts_climbing_at_boundary() {
    let mut terrain_fixture = TerrainFixtureBuilder::new(20, 20)
        .with_row(0, 10, 20, TerrainProps::new())
        .with_col(1, 10, 10, TerrainProps::new())
        .build();

    let mut objects_fixture: ObjectsState = Vec::new();

    let lemming_fixture = LemmingState {
        climber: true,
        ..LemmingState::fixture(0, 10, Direction::Right, Activity::Walking)
    };
    let mut lemming = lemming_fixture.clone();

    lemming.tick(&mut terrain_fixture, &mut objects_fixture);

    assert_eq!(
        lemming,
        LemmingState {
            x: 1,
            activity: Activity::Climbing,
            animation: Some(LemmingAnimation::Climbing),
            frame: 0,
            ..lemming_fixture
        }
    );
}

#[test]
fn climber_does_nothing_without_ledge_in_frame_0_to_2() {
    for frame in 0..=2 {
        let mut terrain_fixture = TerrainFixtureBuilder::new(20, 20)
            .with_row(0, 19, 20, TerrainProps::new())
            .with_col(1, 19, 20, TerrainProps::new())
            .build();

        let mut objects_fixture: ObjectsState = Vec::new();

        let lemming_fixture = LemmingState {
            climber: true,
            frame,
            ..LemmingState::fixture(1, 19, Direction::Right, Activity::Climbing)
        };
        let mut lemming = lemming_fixture.clone();

        lemming.tick(&mut terrain_fixture, &mut objects_fixture);

        assert_eq!(
            lemming,
            LemmingState {
                frame: frame + 1,
                ..lemming_fixture
            }
        );
    }
}

fn test_climber_gropes(frame: usize, delta: u32) {
    let mut terrain_fixture = TerrainFixtureBuilder::new(20, 20)
        .with_row(0, 19, 20, TerrainProps::new())
        .with_col(1, 19, 7 + delta, TerrainProps::new())
        .build();

    let mut objects_fixture: ObjectsState = Vec::new();

    let lemming_fixture = LemmingState {
        climber: true,
        frame,
        ..LemmingState::fixture(1, 19, Direction::Right, Activity::Climbing)
    };
    let mut lemming = lemming_fixture.clone();

    lemming.tick(&mut terrain_fixture, &mut objects_fixture);

    assert_eq!(
        lemming,
        LemmingState {
            frame: 0,
            y: 21 - delta as i32,
            activity: Activity::Hoisting,
            animation: Some(LemmingAnimation::Hoisting),
            ..lemming_fixture
        }
    );
}

fn test_climber_does_not_grope(frame: usize, delta: u32) {
    let mut terrain_fixture = TerrainFixtureBuilder::new(20, 20)
        .with_row(0, 19, 20, TerrainProps::new())
        .with_col(1, 19, 8 + delta, TerrainProps::new())
        .build();

    let mut objects_fixture: ObjectsState = Vec::new();

    let lemming_fixture = LemmingState {
        climber: true,
        frame,
        ..LemmingState::fixture(1, 19, Direction::Right, Activity::Climbing)
    };
    let mut lemming = lemming_fixture.clone();

    lemming.tick(&mut terrain_fixture, &mut objects_fixture);

    assert_eq!(lemming.activity, Activity::Climbing);
}

#[test]
fn climber_gropes_8_pixels_in_frame_0() {
    test_climber_gropes(0, 1);
    test_climber_does_not_grope(0, 2);
}

#[test]
fn climber_gropes_9_pixels_in_frame_1() {
    test_climber_gropes(1, 2);
    test_climber_does_not_grope(1, 3);
}

#[test]
fn climber_gropes_10_pixels_in_frame_2() {
    test_climber_gropes(2, 3);
    test_climber_does_not_grope(2, 4);
}

#[test]
fn climber_gropes_10_pixels_in_frame_7() {
    test_climber_gropes(7, 0);
    test_climber_does_not_grope(7, 1);
}

#[test]
fn climber_groping_into_the_top_turns_and_resets() {
    let mut terrain_fixture = TerrainFixtureBuilder::new(20, 20).build();

    let mut objects_fixture: ObjectsState = Vec::new();

    let lemming_fixture = LemmingState {
        climber: true,
        frame: 1,
        ..LemmingState::fixture(1, 5, Direction::Right, Activity::Climbing)
    };
    let mut lemming = lemming_fixture.clone();

    lemming.tick(&mut terrain_fixture, &mut objects_fixture);

    assert_eq!(
        lemming,
        LemmingState {
            frame: 0,
            y: 5,
            activity: Activity::Hoisting,
            direction: Direction::Left,
            animation: Some(LemmingAnimation::Hoisting),
            ..lemming_fixture
        }
    );
}

#[test]
fn climber_ascends_on_all_other_frames() {
    for frame in 3..=6 {
        let mut terrain_fixture = TerrainFixtureBuilder::new(20, 20).build();

        let mut objects_fixture: ObjectsState = Vec::new();

        let lemming_fixture = LemmingState {
            climber: true,
            frame,
            ..LemmingState::fixture(1, 10, Direction::Right, Activity::Climbing)
        };
        let mut lemming = lemming_fixture.clone();

        lemming.tick(&mut terrain_fixture, &mut objects_fixture);

        assert_eq!(
            lemming,
            LemmingState {
                y: 9,
                frame: frame + 1,
                ..lemming_fixture
            }
        );
    }
}

#[test]
fn climber_right_ascending_into_the_ceiling_turns_and_falls() {
    for frame in 3..=6 {
        let mut terrain_fixture = TerrainFixtureBuilder::new(20, 20).build();

        let mut objects_fixture: ObjectsState = Vec::new();

        let lemming_fixture = LemmingState {
            climber: true,
            frame,
            ..LemmingState::fixture(3, 7, Direction::Right, Activity::Climbing)
        };
        let mut lemming = lemming_fixture.clone();

        lemming.tick(&mut terrain_fixture, &mut objects_fixture);

        assert_eq!(
            lemming,
            LemmingState {
                x: 1,
                direction: Direction::Left,
                y: 6,
                frame: 0,
                activity: Activity::Falling(Default::default()),
                animation: Some(LemmingAnimation::Falling),
                ..lemming_fixture
            }
        );
    }
}

#[test]
fn climber_left_ascending_into_the_ceiling_turns_and_falls() {
    for frame in 3..=6 {
        let mut terrain_fixture = TerrainFixtureBuilder::new(20, 20).build();

        let mut objects_fixture: ObjectsState = Vec::new();

        let lemming_fixture = LemmingState {
            climber: true,
            frame,
            ..LemmingState::fixture(3, 7, Direction::Left, Activity::Climbing)
        };
        let mut lemming = lemming_fixture.clone();

        lemming.tick(&mut terrain_fixture, &mut objects_fixture);

        assert_eq!(
            lemming,
            LemmingState {
                x: 5,
                direction: Direction::Right,
                y: 6,
                frame: 0,
                activity: Activity::Falling(Default::default()),
                animation: Some(LemmingAnimation::Falling),
                ..lemming_fixture
            }
        );
    }
}

#[test]
fn climber_right_ascending_into_a_ledge_turns_and_falls() {
    for frame in 3..=6 {
        let mut terrain_fixture = TerrainFixtureBuilder::new(20, 20)
            .with(2, 10, TerrainProps::new().with_solid(true))
            .build();

        let mut objects_fixture: ObjectsState = Vec::new();

        let lemming_fixture = LemmingState {
            climber: true,
            frame,
            ..LemmingState::fixture(3, 19, Direction::Right, Activity::Climbing)
        };
        let mut lemming = lemming_fixture.clone();

        lemming.tick(&mut terrain_fixture, &mut objects_fixture);

        assert_eq!(
            lemming,
            LemmingState {
                x: 1,
                direction: Direction::Left,
                y: 18,
                frame: 0,
                activity: Activity::Falling(Default::default()),
                animation: Some(LemmingAnimation::Falling),
                ..lemming_fixture
            }
        );
    }
}

#[test]
fn climber_left_ascending_into_a_ledge_turns_and_falls() {
    for frame in 3..=6 {
        let mut terrain_fixture = TerrainFixtureBuilder::new(20, 20)
            .with(4, 10, TerrainProps::new().with_solid(true))
            .build();

        let mut objects_fixture: ObjectsState = Vec::new();

        let lemming_fixture = LemmingState {
            climber: true,
            frame,
            ..LemmingState::fixture(3, 19, Direction::Left, Activity::Climbing)
        };
        let mut lemming = lemming_fixture.clone();

        lemming.tick(&mut terrain_fixture, &mut objects_fixture);

        assert_eq!(
            lemming,
            LemmingState {
                x: 5,
                direction: Direction::Right,
                y: 18,
                frame: 0,
                activity: Activity::Falling(Default::default()),
                animation: Some(LemmingAnimation::Falling),
                ..lemming_fixture
            }
        );
    }
}

#[test]
fn hoister_ascends_in_frames_1_to_3() {
    for frame in 1..=3 {
        let mut terrain_fixture = TerrainFixtureBuilder::new(20, 20).build();

        let mut objects_fixture: ObjectsState = Vec::new();

        let lemming_fixture = LemmingState {
            climber: true,
            frame,
            ..LemmingState::fixture(5, 10, Direction::Right, Activity::Hoisting)
        };
        let mut lemming = lemming_fixture.clone();

        lemming.tick(&mut terrain_fixture, &mut objects_fixture);

        assert_eq!(
            lemming,
            LemmingState {
                y: 8,
                frame: frame + 1,
                ..lemming_fixture
            }
        );
    }
}

#[test]
fn hoister_turns_if_it_ascends_into_the_ceiling_in_frames_1_to_3() {
    for frame in 1..=3 {
        let mut terrain_fixture = TerrainFixtureBuilder::new(20, 20).build();

        let mut objects_fixture: ObjectsState = Vec::new();

        let lemming_fixture = LemmingState {
            climber: true,
            frame,
            ..LemmingState::fixture(5, 8, Direction::Right, Activity::Hoisting)
        };
        let mut lemming = lemming_fixture.clone();

        lemming.tick(&mut terrain_fixture, &mut objects_fixture);

        assert_eq!(
            lemming,
            LemmingState {
                y: 5,
                direction: Direction::Left,
                frame: frame + 1,
                ..lemming_fixture
            }
        );
    }
}

#[test]
fn hoister_does_nothing_on_frames_4_to_6() {
    for frame in 4..=6 {
        let mut terrain_fixture = TerrainFixtureBuilder::new(20, 20).build();

        let mut objects_fixture: ObjectsState = Vec::new();

        let lemming_fixture = LemmingState {
            climber: true,
            frame,
            ..LemmingState::fixture(5, 10, Direction::Right, Activity::Hoisting)
        };
        let mut lemming = lemming_fixture.clone();

        lemming.tick(&mut terrain_fixture, &mut objects_fixture);

        assert_eq!(
            lemming,
            LemmingState {
                frame: frame + 1,
                ..lemming_fixture
            }
        );
    }
}

#[test]
fn hoister_transitions_to_walking_in_frame_7() {
    let mut terrain_fixture = TerrainFixtureBuilder::new(20, 20).build();

    let mut objects_fixture: ObjectsState = Vec::new();

    let lemming_fixture = LemmingState {
        climber: true,
        frame: 7,
        ..LemmingState::fixture(5, 10, Direction::Right, Activity::Hoisting)
    };
    let mut lemming = lemming_fixture.clone();

    lemming.tick(&mut terrain_fixture, &mut objects_fixture);

    assert_eq!(
        lemming,
        LemmingState {
            frame: 0,
            activity: Activity::Walking,
            animation: Some(LemmingAnimation::Walking),
            ..lemming_fixture
        }
    );
}

#[test]
fn hoister_turns_and_walks_if_it_hits_the_ceiling_in_frame_7() {
    let mut terrain_fixture = TerrainFixtureBuilder::new(20, 20).build();

    let mut objects_fixture: ObjectsState = Vec::new();

    let lemming_fixture = LemmingState {
        climber: true,
        frame: 7,
        ..LemmingState::fixture(5, 4, Direction::Right, Activity::Hoisting)
    };
    let mut lemming = lemming_fixture.clone();

    lemming.tick(&mut terrain_fixture, &mut objects_fixture);

    assert_eq!(
        lemming,
        LemmingState {
            y: 3,
            direction: Direction::Left,
            frame: 0,
            activity: Activity::Walking,
            animation: Some(LemmingAnimation::Walking),
            ..lemming_fixture
        }
    );
}
