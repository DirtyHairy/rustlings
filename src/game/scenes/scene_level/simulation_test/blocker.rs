use rustlings::game_data::Skill::{self, Blocker};

use crate::{
    scenes::scene_level::simulation::{
        LemmingVerdict, SelectionResult, test::fixture::TerrainFixtureBuilder,
    },
    state::{Activity, Direction, LemmingAnimation, LemmingState, ObjectState, TerrainProps},
};

#[test]
fn blocker_creates_a_block_zone() {
    let mut terrain = TerrainFixtureBuilder::new(20, 20).build();

    let fixture = LemmingState {
        activity: Activity::Walking,
        x: 9,
        y: 9,
        ..Default::default()
    };
    let mut lemming = fixture.clone();

    let result = lemming.assign_skill(&mut terrain, Blocker);

    assert_eq!(result, SelectionResult::Success);
    assert_eq!(
        lemming,
        LemmingState {
            activity: Activity::Blocking,
            animation: Some(LemmingAnimation::Blocking),
            ..fixture
        }
    );
    assert!(terrain.block_field_overlaps(9, 9));
}

#[test]
fn block_field_left_turns_walker_right() {
    let mut terrain = TerrainFixtureBuilder::new(20, 20)
        .with_row(0, 9, 20, TerrainProps::new())
        .build();
    terrain.create_block_field(9, 9);

    let mut objects: Vec<ObjectState> = Vec::new();

    let fixture = LemmingState {
        activity: Activity::Walking,
        animation: Some(LemmingAnimation::Walking),
        direction: Direction::Right,
        x: 6,
        y: 9,
        ..Default::default()
    };
    let mut lemming = fixture.clone();

    let result = lemming.tick(&mut terrain, &mut objects);

    assert_eq!(result, LemmingVerdict::Continue);
    assert_eq!(
        lemming,
        LemmingState {
            frame: 1,
            x: 7,
            direction: Direction::Left,
            ..fixture
        }
    );
}

#[test]
fn block_field_left_does_not_turn_walker_left() {
    let mut terrain = TerrainFixtureBuilder::new(20, 20)
        .with_row(0, 9, 20, TerrainProps::new())
        .build();
    terrain.create_block_field(9, 9);

    let mut objects: Vec<ObjectState> = Vec::new();

    let fixture = LemmingState {
        activity: Activity::Walking,
        animation: Some(LemmingAnimation::Walking),
        direction: Direction::Left,
        x: 6,
        y: 9,
        ..Default::default()
    };
    let mut lemming = fixture.clone();

    let result = lemming.tick(&mut terrain, &mut objects);

    assert_eq!(result, LemmingVerdict::Continue);
    assert_eq!(
        lemming,
        LemmingState {
            frame: 1,
            x: 5,
            ..fixture
        }
    );
}

#[test]
fn block_field_right_turns_walker_left() {
    let mut terrain = TerrainFixtureBuilder::new(20, 20)
        .with_row(0, 9, 20, TerrainProps::new())
        .build();
    terrain.create_block_field(9, 9);

    let mut objects: Vec<ObjectState> = Vec::new();

    let fixture = LemmingState {
        activity: Activity::Walking,
        animation: Some(LemmingAnimation::Walking),
        direction: Direction::Left,
        x: 14,
        y: 9,
        ..Default::default()
    };
    let mut lemming = fixture.clone();

    let result = lemming.tick(&mut terrain, &mut objects);

    assert_eq!(result, LemmingVerdict::Continue);
    assert_eq!(
        lemming,
        LemmingState {
            frame: 1,
            x: 13,
            direction: Direction::Right,
            ..fixture
        }
    );
}

#[test]
fn block_field_right_does_not_turn_walker_right() {
    let mut terrain = TerrainFixtureBuilder::new(20, 20)
        .with_row(0, 9, 20, TerrainProps::new())
        .build();
    terrain.create_block_field(9, 9);

    let mut objects: Vec<ObjectState> = Vec::new();

    let fixture = LemmingState {
        activity: Activity::Walking,
        animation: Some(LemmingAnimation::Walking),
        direction: Direction::Right,
        x: 14,
        y: 9,
        ..Default::default()
    };
    let mut lemming = fixture.clone();

    let result = lemming.tick(&mut terrain, &mut objects);

    assert_eq!(result, LemmingVerdict::Continue);
    assert_eq!(
        lemming,
        LemmingState {
            frame: 1,
            x: 15,
            ..fixture
        }
    );
}

#[test]
fn blocker_without_ground_turns_into_walker() {
    let mut terrain = TerrainFixtureBuilder::new(20, 20).build();
    let mut objects: Vec<ObjectState> = Vec::new();

    let fixture = LemmingState {
        activity: Activity::Walking,
        animation: Some(LemmingAnimation::Walking),
        x: 9,
        y: 9,
        ..Default::default()
    };
    let mut lemming = fixture.clone();

    let assign_result = lemming.assign_skill(&mut terrain, Skill::Blocker);
    assert_eq!(assign_result, SelectionResult::Success);

    let tick_result = lemming.tick(&mut terrain, &mut objects);

    assert_eq!(tick_result, LemmingVerdict::Continue);
    assert_eq!(
        lemming,
        LemmingState {
            activity: Activity::Walking,
            animation: Some(LemmingAnimation::Walking),
            ..fixture
        }
    );
    assert!(!terrain.block_field_overlaps(9, 9));
}
