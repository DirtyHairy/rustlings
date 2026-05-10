use crate::{
    scenes::scene_level::simulation::{
        CEILING_HIT_Y_RESET, JUMP_DISTANCE, MIN_FOOT_Y, test::fixture::TerrainFixtureBuilder,
    },
    state::{Activity, Direction, LemmingAnimation, LemmingState, ObjectState, TerrainProps},
};

#[test]
fn jumper_continues() {
    let terrain_fixture = TerrainFixtureBuilder::new(20, 20)
        .with_col(10, 15, 4, TerrainProps::new())
        .build();

    let mut objects_fixture: Vec<ObjectState> = Vec::new();

    let lemming_fixture = LemmingState::fixture(10, 15, Direction::Right, Activity::Jumping);
    let mut lemming = lemming_fixture.clone();

    lemming.tick(&terrain_fixture, &mut objects_fixture);

    assert_eq!(
        lemming,
        LemmingState {
            y: 15 - JUMP_DISTANCE as i32,
            ..lemming_fixture
        }
    );
}

#[test]
fn jumper_continues_exact() {
    let terrain_fixture = TerrainFixtureBuilder::new(20, 20)
        .with_col(10, 15, 3, TerrainProps::new())
        .build();

    let mut objects_fixture: Vec<ObjectState> = Vec::new();

    let lemming_fixture = LemmingState::fixture(10, 15, Direction::Right, Activity::Jumping);
    let mut lemming = lemming_fixture.clone();

    lemming.tick(&terrain_fixture, &mut objects_fixture);

    assert_eq!(
        lemming,
        LemmingState {
            y: 15 - JUMP_DISTANCE as i32,
            ..lemming_fixture
        }
    );
}

#[test]
fn jumper_clears_obstacle() {
    let terrain_fixture = TerrainFixtureBuilder::new(20, 20)
        .with_col(10, 15, 2, TerrainProps::new())
        .build();

    let mut objects_fixture: Vec<ObjectState> = Vec::new();

    let lemming_fixture = LemmingState::fixture(10, 15, Direction::Right, Activity::Jumping);
    let mut lemming = lemming_fixture.clone();

    lemming.tick(&terrain_fixture, &mut objects_fixture);

    assert_eq!(
        lemming,
        LemmingState {
            y: 14,
            frame: 0,
            activity: Activity::Walking,
            animation: LemmingAnimation::Walking,
            ..lemming_fixture
        }
    );
}

#[test]
fn jumper_clears_obstacle_exact() {
    let terrain_fixture = TerrainFixtureBuilder::new(20, 20)
        .with_col(10, 15, 1, TerrainProps::new())
        .build();

    let mut objects_fixture: Vec<ObjectState> = Vec::new();

    let lemming_fixture = LemmingState::fixture(10, 15, Direction::Right, Activity::Jumping);
    let mut lemming = lemming_fixture.clone();

    lemming.tick(&terrain_fixture, &mut objects_fixture);

    assert_eq!(
        lemming,
        LemmingState {
            frame: 0,
            activity: Activity::Walking,
            animation: LemmingAnimation::Walking,
            ..lemming_fixture
        }
    );
}

#[test]
fn jumper_hits_ceiling() {
    let terrain_fixture = TerrainFixtureBuilder::new(20, 20)
        .with_col(10, MIN_FOOT_Y as u32 + 1, 3, TerrainProps::new())
        .build();

    let mut objects_fixture: Vec<ObjectState> = Vec::new();

    let lemming_fixture =
        LemmingState::fixture(10, MIN_FOOT_Y + 1, Direction::Right, Activity::Jumping);
    let mut lemming = lemming_fixture.clone();

    lemming.tick(&terrain_fixture, &mut objects_fixture);

    assert_eq!(
        lemming,
        LemmingState {
            y: CEILING_HIT_Y_RESET,
            frame: 0,
            direction: Direction::Left,
            activity: Activity::Walking,
            animation: LemmingAnimation::Walking,
            ..lemming_fixture
        }
    );
}

#[test]
fn jumper_touches_ceiling() {
    let terrain_fixture = TerrainFixtureBuilder::new(20, 20)
        .with_col(10, MIN_FOOT_Y as u32 + 1, 2, TerrainProps::new())
        .build();

    let mut objects_fixture: Vec<ObjectState> = Vec::new();

    let lemming_fixture =
        LemmingState::fixture(10, MIN_FOOT_Y + 1, Direction::Right, Activity::Jumping);
    let mut lemming = lemming_fixture.clone();

    lemming.tick(&terrain_fixture, &mut objects_fixture);

    assert_eq!(
        lemming,
        LemmingState {
            y: MIN_FOOT_Y,
            frame: 0,
            activity: Activity::Walking,
            animation: LemmingAnimation::Walking,
            ..lemming_fixture
        }
    );
}
