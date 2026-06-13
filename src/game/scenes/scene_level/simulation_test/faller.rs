use crate::{
    scenes::scene_level::simulation::{
        FALL_DISTANCE_FLOAT, FALL_DISTANCE_PER_FRAME, FALL_DISTANCE_START_OFFSET, MAX_SAFE_FALL,
        test::fixture::TerrainFixtureBuilder,
    },
    state::{
        Activity, ActivityStateFalling, ActivityStateFloating, Direction, LemmingAnimation,
        LemmingState, ObjectState, TerrainProps,
    },
};

#[test]
fn faller_falls() {
    let mut terrain_fixture = TerrainFixtureBuilder::new(20, 20)
        .with(10, 13, TerrainProps::new())
        .build();

    let mut objects_fixture: Vec<ObjectState> = Vec::new();

    let lemming_fixture = LemmingState::fixture(
        10,
        10,
        Direction::Right,
        Activity::Falling(Default::default()),
    );
    let mut lemming = lemming_fixture.clone();

    lemming.tick(&mut terrain_fixture, &mut objects_fixture);

    assert_eq!(
        lemming,
        LemmingState {
            y: 10 + FALL_DISTANCE_PER_FRAME as i32,
            frame: 1,
            activity: Activity::Falling(ActivityStateFalling {
                delta_y: FALL_DISTANCE_PER_FRAME + FALL_DISTANCE_START_OFFSET
            }),
            ..lemming_fixture
        }
    );
}

#[test]
fn faller_lands_safely() {
    let mut terrain_fixture = TerrainFixtureBuilder::new(20, 20)
        .with(10, 12, TerrainProps::new())
        .build();

    let mut objects_fixture: Vec<ObjectState> = Vec::new();

    let lemming_fixture = LemmingState::fixture(
        10,
        10,
        Direction::Right,
        Activity::Falling(ActivityStateFalling {
            delta_y: MAX_SAFE_FALL,
        }),
    );
    let mut lemming = lemming_fixture.clone();

    lemming.tick(&mut terrain_fixture, &mut objects_fixture);

    assert_eq!(
        lemming,
        LemmingState {
            y: 12,
            frame: 0,
            activity: Activity::Walking,
            animation: LemmingAnimation::Walking,
            ..lemming_fixture
        }
    );
}

#[test]
fn faller_splats() {
    let mut terrain_fixture = TerrainFixtureBuilder::new(20, 20)
        .with(10, 12, TerrainProps::new())
        .build();

    let mut objects_fixture: Vec<ObjectState> = Vec::new();

    let lemming_fixture = LemmingState::fixture(
        10,
        10,
        Direction::Right,
        Activity::Falling(ActivityStateFalling {
            delta_y: MAX_SAFE_FALL + 1,
        }),
    );
    let mut lemming = lemming_fixture.clone();

    lemming.tick(&mut terrain_fixture, &mut objects_fixture);

    assert_eq!(
        lemming,
        LemmingState {
            y: 12,
            frame: 0,
            activity: Activity::Splatting,
            animation: LemmingAnimation::Splatting,
            ..lemming_fixture
        }
    );
}

#[test]
fn faller_with_floater_transitions_to_floating() {
    let mut terrain_fixture = TerrainFixtureBuilder::new(20, 20).build();

    let mut objects_fixture: Vec<ObjectState> = Vec::new();

    let lemming_fixture = LemmingState {
        floater: true,
        ..LemmingState::fixture(
            10,
            10,
            Direction::Right,
            Activity::Falling(ActivityStateFalling {
                delta_y: FALL_DISTANCE_FLOAT,
            }),
        )
    };
    let mut lemming = lemming_fixture.clone();

    lemming.tick(&mut terrain_fixture, &mut objects_fixture);

    assert_eq!(
        lemming,
        LemmingState {
            activity: Activity::Floating(ActivityStateFloating::default()),
            animation: LemmingAnimation::PreUmbrella,
            ..lemming_fixture
        }
    );
}

#[test]
fn faller_with_floater_below_threshold_keeps_falling() {
    let mut terrain_fixture = TerrainFixtureBuilder::new(20, 20).build();

    let mut objects_fixture: Vec<ObjectState> = Vec::new();

    let lemming_fixture = LemmingState {
        floater: true,
        ..LemmingState::fixture(
            10,
            10,
            Direction::Right,
            Activity::Falling(ActivityStateFalling {
                delta_y: FALL_DISTANCE_FLOAT - 1,
            }),
        )
    };
    let mut lemming = lemming_fixture.clone();

    lemming.tick(&mut terrain_fixture, &mut objects_fixture);

    assert_eq!(
        lemming,
        LemmingState {
            y: 10 + FALL_DISTANCE_PER_FRAME as i32,
            frame: 1,
            activity: Activity::Falling(ActivityStateFalling {
                delta_y: FALL_DISTANCE_FLOAT - 1 + FALL_DISTANCE_PER_FRAME,
            }),
            ..lemming_fixture
        }
    );
}

#[test]
fn faller_without_floater_does_not_float() {
    let mut terrain_fixture = TerrainFixtureBuilder::new(20, 20).build();

    let mut objects_fixture: Vec<ObjectState> = Vec::new();

    let lemming_fixture = LemmingState::fixture(
        10,
        10,
        Direction::Right,
        Activity::Falling(ActivityStateFalling {
            delta_y: FALL_DISTANCE_FLOAT,
        }),
    );
    let mut lemming = lemming_fixture.clone();

    lemming.tick(&mut terrain_fixture, &mut objects_fixture);

    assert_eq!(
        lemming,
        LemmingState {
            y: 10 + FALL_DISTANCE_PER_FRAME as i32,
            frame: 1,
            activity: Activity::Falling(ActivityStateFalling {
                delta_y: FALL_DISTANCE_FLOAT + FALL_DISTANCE_PER_FRAME,
            }),
            ..lemming_fixture
        }
    );
}

#[test]
fn faller_wraps_animation() {
    let mut terrain_fixture = TerrainFixtureBuilder::new(20, 20).build();

    let mut objects_fixture: Vec<ObjectState> = Vec::new();

    let lemming_fixture = LemmingState {
        frame: LemmingAnimation::Falling.frame_count() - 1,
        ..LemmingState::fixture(
            10,
            10,
            Direction::Right,
            Activity::Falling(Default::default()),
        )
    };
    let mut lemming = lemming_fixture.clone();

    lemming.tick(&mut terrain_fixture, &mut objects_fixture);

    assert_eq!(
        lemming,
        LemmingState {
            y: 10 + FALL_DISTANCE_PER_FRAME as i32,
            frame: 0,
            activity: Activity::Falling(ActivityStateFalling {
                delta_y: FALL_DISTANCE_PER_FRAME + FALL_DISTANCE_START_OFFSET
            }),
            ..lemming_fixture
        }
    );
}
