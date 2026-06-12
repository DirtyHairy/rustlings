use crate::{
    scenes::scene_level::simulation::{
        DROWNER_MIN_WALL_DISTANCE, LemmingVerdict, test::fixture::TerrainFixtureBuilder,
    },
    state::{Activity, Direction, LemmingAnimation, LemmingState, ObjectState, TerrainProps},
};

#[test]
fn drowner_moves_in_water_right() {
    let mut terrain_fixture = TerrainFixtureBuilder::new(20, 20).build();

    let mut objects_fixture: Vec<ObjectState> = Vec::new();

    let lemming_fixture = LemmingState::fixture(10, 10, Direction::Right, Activity::Drowning);
    let mut lemming = lemming_fixture.clone();

    let verdict = lemming.tick(&mut terrain_fixture, &mut objects_fixture);

    assert_eq!(verdict, LemmingVerdict::Continue);
    assert_eq!(
        lemming,
        LemmingState {
            x: 11,
            frame: 1,
            ..lemming_fixture
        }
    );
}

#[test]
fn drowner_moves_in_water_left() {
    let mut terrain_fixture = TerrainFixtureBuilder::new(20, 20).build();

    let mut objects_fixture: Vec<ObjectState> = Vec::new();

    let lemming_fixture = LemmingState::fixture(10, 10, Direction::Left, Activity::Drowning);
    let mut lemming = lemming_fixture.clone();

    let verdict = lemming.tick(&mut terrain_fixture, &mut objects_fixture);

    assert_eq!(verdict, LemmingVerdict::Continue);
    assert_eq!(
        lemming,
        LemmingState {
            x: 9,
            frame: 1,
            ..lemming_fixture
        }
    );
}

#[test]
fn drowner_stops_at_wall() {
    let mut terrain_fixture = TerrainFixtureBuilder::new(20, 20)
        .with(10 + DROWNER_MIN_WALL_DISTANCE, 10, TerrainProps::new())
        .build();

    let mut objects_fixture: Vec<ObjectState> = Vec::new();

    let lemming_fixture = LemmingState::fixture(10, 10, Direction::Right, Activity::Drowning);
    let mut lemming = lemming_fixture.clone();

    let verdict = lemming.tick(&mut terrain_fixture, &mut objects_fixture);

    assert_eq!(verdict, LemmingVerdict::Continue);
    assert_eq!(
        lemming,
        LemmingState {
            frame: 1,
            ..lemming_fixture
        }
    );
}

#[test]
fn drowner_advances_towards_wall() {
    let mut terrain_fixture = TerrainFixtureBuilder::new(20, 20)
        .with(10 + DROWNER_MIN_WALL_DISTANCE + 1, 10, TerrainProps::new())
        .build();

    let mut objects_fixture: Vec<ObjectState> = Vec::new();

    let lemming_fixture = LemmingState::fixture(10, 10, Direction::Right, Activity::Drowning);
    let mut lemming = lemming_fixture.clone();

    let verdict = lemming.tick(&mut terrain_fixture, &mut objects_fixture);

    assert_eq!(verdict, LemmingVerdict::Continue);
    assert_eq!(
        lemming,
        LemmingState {
            x: 11,
            frame: 1,
            ..lemming_fixture
        }
    );
}

#[test]
fn drowner_removed_after_animation() {
    let mut terrain_fixture = TerrainFixtureBuilder::new(20, 20).build();

    let mut objects_fixture: Vec<ObjectState> = Vec::new();

    let lemming_fixture = LemmingState {
        frame: LemmingAnimation::Drowning.frame_count() - 1,
        ..LemmingState::fixture(10, 10, Direction::Right, Activity::Drowning)
    };
    let mut lemming = lemming_fixture.clone();

    let verdict = lemming.tick(&mut terrain_fixture, &mut objects_fixture);

    assert_eq!(verdict, LemmingVerdict::Death);
}
