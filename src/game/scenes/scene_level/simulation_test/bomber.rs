use rustlings::game_data::NUM_PARTICLE_SETS;

use crate::{
    scenes::scene_level::simulation::{LemmingVerdict, test::fixture::TerrainFixtureBuilder},
    state::{
        Activity, ActivityStateFalling, ActivityStateFloating, Direction, LemmingAnimation,
        LemmingHealth::{self, Exploding},
        LemmingState, ObjectState, TerrainProps,
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
            frame: 0,
            health: LemmingHealth::OhNo,
            countdown: None,
            animation: Some(LemmingAnimation::OhNo),
            ..lemming_fixture
        }
    );
}

#[test]
fn faller_transitions_to_explosion_after_countdown() {
    let mut terrain_fixture = TerrainFixtureBuilder::new(20, 20)
        .with_row(0, 10, 20, TerrainProps::new())
        .build();

    let mut objects_fixture: Vec<ObjectState> = Vec::new();

    let lemming_fixture = LemmingState {
        countdown: Some(0),
        ..LemmingState::fixture(
            5,
            10,
            Direction::Right,
            Activity::Falling(ActivityStateFalling { delta_y: 0 }),
        )
    };

    let mut lemming = lemming_fixture.clone();

    lemming.tick(&mut terrain_fixture, &mut objects_fixture);

    assert_eq!(
        lemming,
        LemmingState {
            frame: 0,
            health: LemmingHealth::Exploding,
            countdown: None,
            animation: Some(LemmingAnimation::Explosion),
            ..lemming_fixture
        }
    );
}

#[test]
fn floater_transitions_to_explosion_after_countdown() {
    let mut terrain_fixture = TerrainFixtureBuilder::new(20, 20)
        .with_row(0, 10, 20, TerrainProps::new())
        .build();

    let mut objects_fixture: Vec<ObjectState> = Vec::new();

    let lemming_fixture = LemmingState {
        countdown: Some(0),
        ..LemmingState::fixture(
            5,
            10,
            Direction::Right,
            Activity::Floating(ActivityStateFloating { tick: 0 }),
        )
    };

    let mut lemming = lemming_fixture.clone();

    lemming.tick(&mut terrain_fixture, &mut objects_fixture);

    assert_eq!(
        lemming,
        LemmingState {
            frame: 0,
            health: LemmingHealth::Exploding,
            countdown: None,
            animation: Some(LemmingAnimation::Explosion),
            ..lemming_fixture
        }
    );
}

#[test]
fn fryer_transitions_to_explosion_after_countdown() {
    let mut terrain_fixture = TerrainFixtureBuilder::new(20, 20)
        .with_row(0, 10, 20, TerrainProps::new())
        .build();

    let mut objects_fixture: Vec<ObjectState> = Vec::new();

    let lemming_fixture = LemmingState {
        countdown: Some(0),
        ..LemmingState::fixture(5, 10, Direction::Right, Activity::Frying)
    };

    let mut lemming = lemming_fixture.clone();

    lemming.tick(&mut terrain_fixture, &mut objects_fixture);

    assert_eq!(
        lemming,
        LemmingState {
            frame: 0,
            health: LemmingHealth::Exploding,
            countdown: None,
            animation: Some(LemmingAnimation::Explosion),
            ..lemming_fixture
        }
    );
}

#[test]
fn exploding_starts_particle_shower_after_first_framse() {
    let mut terrain_fixture = TerrainFixtureBuilder::new(20, 20)
        .with_row(0, 10, 20, TerrainProps::new())
        .build();

    let mut objects_fixture: Vec<ObjectState> = Vec::new();

    let lemming_fixture = LemmingState {
        health: Exploding,
        frame: 0,
        animation: Some(LemmingAnimation::Explosion),
        ..LemmingState::fixture(5, 10, Direction::Right, Activity::Walking)
    };

    let mut lemming = lemming_fixture.clone();

    lemming.tick(&mut terrain_fixture, &mut objects_fixture);

    assert_eq!(
        lemming,
        LemmingState {
            frame: 1,
            animation: None,
            particle_frame: Some(0),
            ..lemming_fixture
        }
    );
}

#[test]
fn exploding_lemming_stays_alive_while_particle_shower_continues() {
    let mut terrain_fixture = TerrainFixtureBuilder::new(20, 20)
        .with_row(0, 10, 20, TerrainProps::new())
        .build();

    let mut objects_fixture: Vec<ObjectState> = Vec::new();

    let lemming_fixture = LemmingState {
        health: Exploding,
        frame: 1,
        animation: None,
        particle_frame: Some(NUM_PARTICLE_SETS - 2),
        ..LemmingState::fixture(5, 10, Direction::Right, Activity::Walking)
    };

    let mut lemming = lemming_fixture.clone();

    assert_eq!(
        lemming.tick(&mut terrain_fixture, &mut objects_fixture),
        LemmingVerdict::Continue,
    );

    assert_eq!(
        lemming,
        LemmingState {
            particle_frame: Some(NUM_PARTICLE_SETS - 1),
            ..lemming_fixture
        }
    );
}

#[test]
fn exploding_lemming_dies_when_particle_shower_stop() {
    let mut terrain_fixture = TerrainFixtureBuilder::new(20, 20)
        .with_row(0, 10, 20, TerrainProps::new())
        .build();

    let mut objects_fixture: Vec<ObjectState> = Vec::new();

    let lemming_fixture = LemmingState {
        health: Exploding,
        frame: 1,
        animation: None,
        particle_frame: Some(NUM_PARTICLE_SETS - 1),
        ..LemmingState::fixture(5, 10, Direction::Right, Activity::Walking)
    };

    let mut lemming = lemming_fixture.clone();

    assert_eq!(
        lemming.tick(&mut terrain_fixture, &mut objects_fixture),
        LemmingVerdict::Death,
    );
}

#[test]
fn ohnoer_falls() {
    let mut terrain_fixture = TerrainFixtureBuilder::new(20, 20).build();

    let mut objects_fixture: Vec<ObjectState> = Vec::new();

    let lemming_fixture = LemmingState {
        health: LemmingHealth::OhNo,
        frame: 0,
        ..LemmingState::fixture(5, 10, Direction::Right, Activity::Walking)
    };

    let mut lemming = lemming_fixture.clone();

    lemming.tick(&mut terrain_fixture, &mut objects_fixture);

    assert_eq!(
        lemming,
        LemmingState {
            y: 13,
            frame: 1,
            ..lemming_fixture
        }
    );
}
