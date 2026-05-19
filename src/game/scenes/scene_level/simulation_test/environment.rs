use rustlings::game_data::LEVEL_HEIGHT;

use crate::{
    scenes::scene_level::simulation::{
        FALL_DISTANCE_PER_FRAME, LemmingVerdict, test::fixture::TerrainFixtureBuilder,
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

    let verdict = lemming.tick(&terrain_fixture, &mut objects_fixture);

    assert_eq!(verdict, LemmingVerdict::Death);
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

    let verdict = lemming.tick(&terrain_fixture, &mut objects_fixture);

    assert_eq!(verdict, LemmingVerdict::Continue);
}

#[test]
fn exit_transitions_walker_to_exiting() {
    let terrain_fixture = TerrainFixtureBuilder::new(20, 20)
        .with(10, 10, TerrainProps::new())
        .with(11, 10, TerrainProps::new().with_exit(true))
        .build();

    let mut objects_fixture: Vec<ObjectState> = Vec::new();

    let lemming_fixture = LemmingState::fixture(10, 10, Direction::Right, Activity::Walking);
    let mut lemming = lemming_fixture.clone();

    let verdict = lemming.tick(&terrain_fixture, &mut objects_fixture);

    assert_eq!(verdict, LemmingVerdict::Continue);
    assert_eq!(
        lemming,
        LemmingState {
            x: 11,
            frame: 0,
            activity: Activity::Exiting,
            animation: LemmingAnimation::Exiting,
            ..lemming_fixture
        }
    );
}

#[test]
fn exit_does_not_trigger_during_fall() {
    let terrain_fixture = TerrainFixtureBuilder::new(20, 20)
        .with_non_solid(10, 13, TerrainProps::new().with_exit(true))
        .build();

    let mut objects_fixture: Vec<ObjectState> = Vec::new();

    let lemming_fixture = LemmingState::fixture(
        10,
        10,
        Direction::Right,
        Activity::Falling(Default::default()),
    );
    let mut lemming = lemming_fixture.clone();

    let verdict = lemming.tick(&terrain_fixture, &mut objects_fixture);

    assert_eq!(verdict, LemmingVerdict::Continue);
    assert!(matches!(lemming.activity, Activity::Falling(_)));
    assert_eq!(lemming.y, 10 + FALL_DISTANCE_PER_FRAME as i32);
    assert_eq!(lemming.animation, LemmingAnimation::Falling);
}

#[test]
fn exiting_advances_animation() {
    let terrain_fixture = TerrainFixtureBuilder::new(20, 20).build();

    let mut objects_fixture: Vec<ObjectState> = Vec::new();

    let lemming_fixture = LemmingState::fixture(10, 10, Direction::Right, Activity::Exiting);
    let mut lemming = lemming_fixture.clone();

    let verdict = lemming.tick(&terrain_fixture, &mut objects_fixture);

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
fn exiting_completes_with_exit_verdict() {
    let terrain_fixture = TerrainFixtureBuilder::new(20, 20).build();

    let mut objects_fixture: Vec<ObjectState> = Vec::new();

    let lemming_fixture = LemmingState {
        frame: LemmingAnimation::Exiting.frame_count() - 1,
        ..LemmingState::fixture(10, 10, Direction::Right, Activity::Exiting)
    };
    let mut lemming = lemming_fixture.clone();

    let verdict = lemming.tick(&terrain_fixture, &mut objects_fixture);

    assert_eq!(verdict, LemmingVerdict::Exit);
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

    let verdict = lemming.tick(&terrain_fixture, &mut objects_fixture);

    assert_eq!(verdict, LemmingVerdict::Death);
}
