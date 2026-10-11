use crate::{
    scenes::scene_level::simulation::test::fixture::TerrainFixtureBuilder,
    state::{
        Activity, Direction, LemmingAnimation, LemmingHealth, LemmingState, ObjectState,
        TerrainProps,
    },
};

#[test]
fn contdown_decrements_each_tick() {
    let mut terrain_fixture = TerrainFixtureBuilder::new(20, 20)
        .with_row(0, 10, 20, TerrainProps::new())
        .build();

    let mut objects_fixture: Vec<ObjectState> = Vec::new();

    let lemming_fixture = LemmingState {
        countdown: Some(10),
        ..LemmingState::fixture(5, 10, Direction::Right, Activity::Walking)
    };

    let mut lemming = lemming_fixture.clone();

    lemming.tick(&mut terrain_fixture, &mut objects_fixture);

    assert_eq!(
        lemming,
        LemmingState {
            x: 6,
            frame: 1,
            countdown: Some(9),
            ..lemming_fixture
        }
    );
}

#[test]
fn walker_transitions_to_ohno_after_countdown() {
    let mut terrain_fixture = TerrainFixtureBuilder::new(20, 20)
        .with_row(0, 10, 20, TerrainProps::new())
        .build();

    let mut objects_fixture: Vec<ObjectState> = Vec::new();

    let lemming_fixture = LemmingState {
        countdown: Some(0),
        ..LemmingState::fixture(5, 10, Direction::Right, Activity::Walking)
    };

    let mut lemming = lemming_fixture.clone();

    lemming.tick(&mut terrain_fixture, &mut objects_fixture);

    assert_eq!(
        lemming,
        LemmingState {
            x: 5,
            frame: 0,
            health: LemmingHealth::OhNo,
            countdown: None,
            animation: Some(LemmingAnimation::OhNo),
            ..lemming_fixture
        }
    );
}
