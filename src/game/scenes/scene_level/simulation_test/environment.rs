use rustlings::game_data::LEVEL_HEIGHT;

use crate::{
    scenes::scene_level::simulation::{
        FALL_DISTANCE_PER_FRAME, test::fixture::TerrainFixtureBuilder,
    },
    state::{Activity, Direction, LemmingAnimation, LemmingState, ObjectState, TerrainProps},
};

#[test]
fn disintegrate_transitions_to_frying() {
    let terrain_fixture = TerrainFixtureBuilder::new(20, 20)
        .with(10, 10, TerrainProps::new())
        .with(11, 10, TerrainProps::new().with_disintegrate(true))
        .build();

    let mut objects_fixture: Vec<ObjectState> = Vec::new();

    let lemming_fixture = LemmingState::fixture(10, 10, Direction::Right, Activity::Walking);
    let mut lemming = lemming_fixture.clone();

    lemming.tick(&terrain_fixture, &mut objects_fixture);

    assert_eq!(
        lemming,
        LemmingState {
            x: 11,
            frame: 0,
            activity: Activity::Frying,
            animation: LemmingAnimation::Frying,
            ..lemming_fixture
        }
    );
}

#[test]
fn drown_transitions_to_drowning() {
    let terrain_fixture = TerrainFixtureBuilder::new(20, 20)
        .with(10, 10, TerrainProps::new())
        .with(11, 10, TerrainProps::new().with_drown(true))
        .build();

    let mut objects_fixture: Vec<ObjectState> = Vec::new();

    let lemming_fixture = LemmingState::fixture(10, 10, Direction::Right, Activity::Walking);
    let mut lemming = lemming_fixture.clone();

    lemming.tick(&terrain_fixture, &mut objects_fixture);

    assert_eq!(
        lemming,
        LemmingState {
            x: 11,
            frame: 0,
            activity: Activity::Drowning,
            animation: LemmingAnimation::Drowning,
            ..lemming_fixture
        }
    );
}

#[test]
fn trap_triggers_and_removes_lemming() {
    let terrain_fixture = TerrainFixtureBuilder::new(20, 20)
        .with(10, 10, TerrainProps::new())
        .with(
            11,
            10,
            TerrainProps::new().with_trap(true).with_object_index(0),
        )
        .build();

    let mut objects_fixture: Vec<ObjectState> = vec![Default::default()];

    let lemming_fixture = LemmingState::fixture(10, 10, Direction::Right, Activity::Walking);
    let mut lemming = lemming_fixture.clone();

    let keep = lemming.tick(&terrain_fixture, &mut objects_fixture);

    assert!(!keep);
    assert!(objects_fixture[0].triggered);
}

#[test]
fn trap_does_not_retrigger() {
    let terrain_fixture = TerrainFixtureBuilder::new(20, 20)
        .with(10, 10, TerrainProps::new())
        .with(
            11,
            10,
            TerrainProps::new().with_trap(true).with_object_index(0),
        )
        .build();

    let mut objects_fixture: Vec<ObjectState> = vec![ObjectState {
        triggered: true,
        ..Default::default()
    }];

    let lemming_fixture = LemmingState::fixture(10, 10, Direction::Right, Activity::Walking);
    let mut lemming = lemming_fixture.clone();

    let keep = lemming.tick(&terrain_fixture, &mut objects_fixture);

    assert!(keep);
}

#[test]
fn lemming_removed_at_bottom() {
    let foot_height = LemmingAnimation::Falling.foot().1;
    let initial_y = (LEVEL_HEIGHT + foot_height - FALL_DISTANCE_PER_FRAME) as i32;

    let terrain_fixture = TerrainFixtureBuilder::new(20, LEVEL_HEIGHT + foot_height + 10).build();

    let mut objects_fixture: Vec<ObjectState> = Vec::new();

    let lemming_fixture = LemmingState::fixture(
        10,
        initial_y,
        Direction::Right,
        Activity::Falling(Default::default()),
    );
    let mut lemming = lemming_fixture.clone();

    let keep = lemming.tick(&terrain_fixture, &mut objects_fixture);

    assert!(!keep);
}
