use crate::{
    scenes::scene_level::simulation::{
        CEILING_HIT_Y_RESET, MAX_JUMP, MAX_STEP_DOWN, MAX_STEP_UP, MIN_FOOT_Y,
        test::fixture::TerrainFixtureBuilder,
    },
    state::{Activity, Direction, LemmingAnimation, LemmingState, ObjectState, TerrainProps},
};

#[test]
fn walker_steps_up_right() {
    for step in 0..=MAX_STEP_UP {
        let mut terrain_fixture = TerrainFixtureBuilder::new(20, 20)
            .with(10, 19, TerrainProps::new())
            .with_col(11, 19, step + 1, TerrainProps::new())
            .build();

        let mut objects_fixture: Vec<ObjectState> = Vec::new();

        let lemming_fixture = LemmingState::fixture(10, 19, Direction::Right, Activity::Walking);
        let mut lemming = lemming_fixture.clone();

        lemming.tick(&mut terrain_fixture, &mut objects_fixture);

        assert_eq!(
            lemming,
            LemmingState {
                x: 11,
                y: 19 - step as i32,
                frame: 1,
                ..lemming_fixture
            },
            "should step {} pixels",
            step
        );
    }
}

#[test]
fn walker_steps_up_left() {
    for step in 0..=MAX_STEP_UP {
        let mut terrain_fixture = TerrainFixtureBuilder::new(20, 20)
            .with(10, 19, TerrainProps::new())
            .with_col(9, 19, step + 1, TerrainProps::new())
            .build();

        let mut objects_fixture: Vec<ObjectState> = Vec::new();

        let lemming_fixture = LemmingState::fixture(10, 19, Direction::Left, Activity::Walking);
        let mut lemming = lemming_fixture.clone();

        lemming.tick(&mut terrain_fixture, &mut objects_fixture);

        assert_eq!(
            lemming,
            LemmingState {
                x: 9,
                y: 19 - step as i32,
                frame: 1,
                ..lemming_fixture
            },
            "should step {} pixels",
            step
        );
    }
}

#[test]
fn walker_jumps_right() {
    for step in MAX_STEP_UP + 1..=MAX_JUMP {
        let mut terrain_fixture = TerrainFixtureBuilder::new(20, 20)
            .with(10, 19, TerrainProps::new())
            .with_col(11, 19, step + 1, TerrainProps::new())
            .build();

        let mut objects_fixture: Vec<ObjectState> = Vec::new();

        let lemming_fixture = LemmingState::fixture(10, 19, Direction::Right, Activity::Walking);
        let mut lemming = lemming_fixture.clone();

        lemming.tick(&mut terrain_fixture, &mut objects_fixture);

        assert_eq!(
            lemming,
            LemmingState {
                x: 11,
                y: 17,
                frame: 0,
                activity: Activity::Jumping,
                animation: LemmingAnimation::Jumping,
                ..lemming_fixture
            },
            "should jump {} pixels",
            step
        );
    }
}

#[test]
fn walker_jumps_left() {
    for step in MAX_STEP_UP + 1..=MAX_JUMP {
        let mut terrain_fixture = TerrainFixtureBuilder::new(20, 20)
            .with(10, 19, TerrainProps::new())
            .with_col(9, 19, step + 1, TerrainProps::new())
            .build();

        let mut objects_fixture: Vec<ObjectState> = Vec::new();

        let lemming_fixture = LemmingState::fixture(10, 19, Direction::Left, Activity::Walking);
        let mut lemming = lemming_fixture.clone();

        lemming.tick(&mut terrain_fixture, &mut objects_fixture);

        assert_eq!(
            lemming,
            LemmingState {
                x: 9,
                y: 17,
                frame: 0,
                activity: Activity::Jumping,
                animation: LemmingAnimation::Jumping,
                ..lemming_fixture
            },
            "should jump {} pixels",
            step
        );
    }
}

#[test]
fn walker_should_turn_left() {
    let mut terrain_fixture = TerrainFixtureBuilder::new(20, 20)
        .with(10, 19, TerrainProps::new())
        .with_col(11, 19, MAX_JUMP + 2, TerrainProps::new())
        .build();

    let mut objects_fixture: Vec<ObjectState> = Vec::new();

    let lemming_fixture = LemmingState::fixture(10, 19, Direction::Right, Activity::Walking);
    let mut lemming = lemming_fixture.clone();

    lemming.tick(&mut terrain_fixture, &mut objects_fixture);

    assert_eq!(
        lemming,
        LemmingState {
            x: 11,
            frame: 1,
            direction: Direction::Left,
            ..lemming_fixture
        }
    );
}

#[test]
fn walker_should_turn_right() {
    let mut terrain_fixture = TerrainFixtureBuilder::new(20, 20)
        .with(10, 19, TerrainProps::new())
        .with_col(9, 19, MAX_JUMP + 2, TerrainProps::new())
        .build();

    let mut objects_fixture: Vec<ObjectState> = Vec::new();

    let lemming_fixture = LemmingState::fixture(10, 19, Direction::Left, Activity::Walking);
    let mut lemming = lemming_fixture.clone();

    lemming.tick(&mut terrain_fixture, &mut objects_fixture);

    assert_eq!(
        lemming,
        LemmingState {
            x: 9,
            frame: 1,
            direction: Direction::Right,
            ..lemming_fixture
        }
    );
}

#[test]
fn walker_wraps_animation() {
    let mut terrain_fixture = TerrainFixtureBuilder::new(20, 20)
        .with_row(10, 19, 2, TerrainProps::new())
        .build();

    let mut objects_fixture: Vec<ObjectState> = Vec::new();

    let lemming_fixture = LemmingState {
        frame: LemmingAnimation::Walking.frame_count() - 1,
        ..LemmingState::fixture(10, 19, Direction::Right, Activity::Walking)
    };
    let mut lemming = lemming_fixture.clone();

    lemming.tick(&mut terrain_fixture, &mut objects_fixture);

    assert_eq!(
        lemming,
        LemmingState {
            x: 11,
            frame: 0,
            ..lemming_fixture
        }
    );
}

#[test]
fn walker_steps_down_right() {
    for step in 1..=MAX_STEP_DOWN {
        let mut terrain_fixture = TerrainFixtureBuilder::new(20, 20)
            .with(10, 10, TerrainProps::new())
            .with(11, 10 + step, TerrainProps::new())
            .build();

        let mut objects_fixture: Vec<ObjectState> = Vec::new();

        let lemming_fixture = LemmingState::fixture(10, 10, Direction::Right, Activity::Walking);
        let mut lemming = lemming_fixture.clone();

        lemming.tick(&mut terrain_fixture, &mut objects_fixture);

        assert_eq!(
            lemming,
            LemmingState {
                x: 11,
                y: (10 + step) as i32,
                frame: 1,
                ..lemming_fixture
            },
            "should step down {} pixels",
            step
        );
    }
}

#[test]
fn walker_steps_down_left() {
    for step in 1..=MAX_STEP_DOWN {
        let mut terrain_fixture = TerrainFixtureBuilder::new(20, 20)
            .with(10, 10, TerrainProps::new())
            .with(9, 10 + step, TerrainProps::new())
            .build();

        let mut objects_fixture: Vec<ObjectState> = Vec::new();

        let lemming_fixture = LemmingState::fixture(10, 10, Direction::Left, Activity::Walking);
        let mut lemming = lemming_fixture.clone();

        lemming.tick(&mut terrain_fixture, &mut objects_fixture);

        assert_eq!(
            lemming,
            LemmingState {
                x: 9,
                y: (10 + step) as i32,
                frame: 1,
                ..lemming_fixture
            },
            "should step down {} pixels",
            step
        );
    }
}

#[test]
fn walker_starts_falling_right() {
    let mut terrain_fixture = TerrainFixtureBuilder::new(20, 20)
        .with(10, 10, TerrainProps::new())
        .with(11, 10 + MAX_STEP_DOWN + 1, TerrainProps::new())
        .build();

    let mut objects_fixture: Vec<ObjectState> = Vec::new();

    let lemming_fixture = LemmingState::fixture(10, 10, Direction::Right, Activity::Walking);
    let mut lemming = lemming_fixture.clone();

    lemming.tick(&mut terrain_fixture, &mut objects_fixture);

    assert_eq!(
        lemming,
        LemmingState {
            x: 11,
            y: 10 + (MAX_STEP_DOWN + 1) as i32,
            frame: 0,
            activity: Activity::Falling(Default::default()),
            animation: LemmingAnimation::Falling,
            ..lemming_fixture
        }
    );
}

#[test]
fn walker_starts_falling_left() {
    let mut terrain_fixture = TerrainFixtureBuilder::new(20, 20)
        .with(10, 10, TerrainProps::new())
        .with(9, 10 + MAX_STEP_DOWN + 1, TerrainProps::new())
        .build();

    let mut objects_fixture: Vec<ObjectState> = Vec::new();

    let lemming_fixture = LemmingState::fixture(10, 10, Direction::Left, Activity::Walking);
    let mut lemming = lemming_fixture.clone();

    lemming.tick(&mut terrain_fixture, &mut objects_fixture);

    assert_eq!(
        lemming,
        LemmingState {
            x: 9,
            y: 10 + (MAX_STEP_DOWN + 1) as i32,
            frame: 0,
            activity: Activity::Falling(Default::default()),
            animation: LemmingAnimation::Falling,
            ..lemming_fixture
        }
    );
}

#[test]
fn walker_turns_at_left_boundary() {
    let mut terrain_fixture = TerrainFixtureBuilder::new(20, 20)
        .with_row(0, 10, 2, TerrainProps::new())
        .build();

    let mut objects_fixture: Vec<ObjectState> = Vec::new();

    let lemming_fixture = LemmingState::fixture(1, 10, Direction::Left, Activity::Walking);
    let mut lemming = lemming_fixture.clone();

    lemming.tick(&mut terrain_fixture, &mut objects_fixture);

    assert_eq!(
        lemming,
        LemmingState {
            x: 0,
            frame: 1,
            direction: Direction::Right,
            ..lemming_fixture
        }
    );
}

#[test]
fn walker_turns_at_right_boundary() {
    let mut terrain_fixture = TerrainFixtureBuilder::new(20, 20)
        .with_row(18, 10, 2, TerrainProps::new())
        .build();

    let mut objects_fixture: Vec<ObjectState> = Vec::new();

    let lemming_fixture = LemmingState::fixture(18, 10, Direction::Right, Activity::Walking);
    let mut lemming = lemming_fixture.clone();

    lemming.tick(&mut terrain_fixture, &mut objects_fixture);

    assert_eq!(
        lemming,
        LemmingState {
            x: 19,
            frame: 1,
            direction: Direction::Left,
            ..lemming_fixture
        }
    );
}

#[test]
fn walker_hits_ceiling_right() {
    let mut terrain_fixture = TerrainFixtureBuilder::new(20, 20)
        .with(10, MIN_FOOT_Y as u32, TerrainProps::new())
        .with_col(11, MIN_FOOT_Y as u32, 2, TerrainProps::new())
        .build();

    let mut objects_fixture: Vec<ObjectState> = Vec::new();

    let lemming_fixture =
        LemmingState::fixture(10, MIN_FOOT_Y, Direction::Right, Activity::Walking);
    let mut lemming = lemming_fixture.clone();

    lemming.tick(&mut terrain_fixture, &mut objects_fixture);

    assert_eq!(
        lemming,
        LemmingState {
            x: 11,
            y: CEILING_HIT_Y_RESET,
            frame: 1,
            direction: Direction::Left,
            ..lemming_fixture
        }
    );
}

#[test]
fn walker_hits_ceiling_left() {
    let mut terrain_fixture = TerrainFixtureBuilder::new(20, 20)
        .with(10, MIN_FOOT_Y as u32, TerrainProps::new())
        .with_col(9, MIN_FOOT_Y as u32, 2, TerrainProps::new())
        .build();

    let mut objects_fixture: Vec<ObjectState> = Vec::new();

    let lemming_fixture = LemmingState::fixture(10, MIN_FOOT_Y, Direction::Left, Activity::Walking);
    let mut lemming = lemming_fixture.clone();

    lemming.tick(&mut terrain_fixture, &mut objects_fixture);

    assert_eq!(
        lemming,
        LemmingState {
            x: 9,
            y: CEILING_HIT_Y_RESET,
            frame: 1,
            direction: Direction::Right,
            ..lemming_fixture
        }
    );
}
