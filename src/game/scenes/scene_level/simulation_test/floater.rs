use crate::{
    scenes::scene_level::simulation::{LemmingVerdict, test::fixture::TerrainFixtureBuilder},
    state::{
        Activity, ActivityStateFloating, Direction, LemmingAnimation, LemmingState, ObjectState,
        TerrainProps,
    },
};

fn floater_at_tick(tick: u32, animation: LemmingAnimation, frame: usize) -> LemmingState {
    LemmingState {
        x: 10,
        y: 10,
        direction: Direction::Right,
        activity: Activity::Floating(ActivityStateFloating { tick }),
        animation,
        frame,
        ..Default::default()
    }
}

#[test]
fn floater_pre_umbrella_advances() {
    let mut terrain_fixture = TerrainFixtureBuilder::new(20, 20).build();

    let mut objects_fixture: Vec<ObjectState> = Vec::new();

    let lemming_fixture = floater_at_tick(0, LemmingAnimation::PreUmbrella, 0);
    let mut lemming = lemming_fixture.clone();

    let verdict = lemming.tick(&mut terrain_fixture, &mut objects_fixture);

    assert_eq!(verdict, LemmingVerdict::Continue);
    assert_eq!(
        lemming,
        LemmingState {
            y: 10 + 3,
            frame: 1,
            activity: Activity::Floating(ActivityStateFloating { tick: 1 }),
            ..lemming_fixture
        }
    );
}

#[test]
fn floater_opens_umbrella() {
    let mut terrain_fixture = TerrainFixtureBuilder::new(20, 20).build();

    let mut objects_fixture: Vec<ObjectState> = Vec::new();

    let lemming_fixture = floater_at_tick(3, LemmingAnimation::PreUmbrella, 3);
    let mut lemming = lemming_fixture.clone();

    let verdict = lemming.tick(&mut terrain_fixture, &mut objects_fixture);

    assert_eq!(verdict, LemmingVerdict::Continue);
    assert_eq!(
        lemming,
        LemmingState {
            y: 10 + 3,
            frame: 1,
            animation: LemmingAnimation::Umbrella,
            activity: Activity::Floating(ActivityStateFloating { tick: 4 }),
            ..lemming_fixture
        }
    );
}

#[test]
fn floater_bounces_up() {
    let mut terrain_fixture = TerrainFixtureBuilder::new(20, 20).build();

    let mut objects_fixture: Vec<ObjectState> = Vec::new();

    let lemming_fixture = floater_at_tick(4, LemmingAnimation::Umbrella, 1);
    let mut lemming = lemming_fixture.clone();

    let verdict = lemming.tick(&mut terrain_fixture, &mut objects_fixture);

    assert_eq!(verdict, LemmingVerdict::Continue);
    assert_eq!(
        lemming,
        LemmingState {
            y: 10 - 1,
            activity: Activity::Floating(ActivityStateFloating { tick: 5 }),
            ..lemming_fixture
        }
    );
}

#[test]
fn floater_hovers() {
    let mut terrain_fixture = TerrainFixtureBuilder::new(20, 20).build();

    let mut objects_fixture: Vec<ObjectState> = Vec::new();

    let lemming_fixture = floater_at_tick(5, LemmingAnimation::Umbrella, 1);
    let mut lemming = lemming_fixture.clone();

    let verdict = lemming.tick(&mut terrain_fixture, &mut objects_fixture);

    assert_eq!(verdict, LemmingVerdict::Continue);
    assert_eq!(
        lemming,
        LemmingState {
            activity: Activity::Floating(ActivityStateFloating { tick: 6 }),
            ..lemming_fixture
        }
    );
}

#[test]
fn floater_slow_descent() {
    let mut terrain_fixture = TerrainFixtureBuilder::new(20, 20).build();

    let mut objects_fixture: Vec<ObjectState> = Vec::new();

    let lemming_fixture = floater_at_tick(6, LemmingAnimation::Umbrella, 1);
    let mut lemming = lemming_fixture.clone();

    let verdict = lemming.tick(&mut terrain_fixture, &mut objects_fixture);

    assert_eq!(verdict, LemmingVerdict::Continue);
    assert_eq!(
        lemming,
        LemmingState {
            y: 10 + 1,
            activity: Activity::Floating(ActivityStateFloating { tick: 7 }),
            ..lemming_fixture
        }
    );
}

#[test]
fn floater_steady_descent() {
    let mut terrain_fixture = TerrainFixtureBuilder::new(20, 20).build();

    let mut objects_fixture: Vec<ObjectState> = Vec::new();

    let lemming_fixture = floater_at_tick(8, LemmingAnimation::Umbrella, 0);
    let mut lemming = lemming_fixture.clone();

    let verdict = lemming.tick(&mut terrain_fixture, &mut objects_fixture);

    assert_eq!(verdict, LemmingVerdict::Continue);
    assert_eq!(
        lemming,
        LemmingState {
            y: 10 + 2,
            frame: 1,
            activity: Activity::Floating(ActivityStateFloating { tick: 9 }),
            ..lemming_fixture
        }
    );
}

#[test]
fn floater_lands_on_ground() {
    let mut terrain_fixture = TerrainFixtureBuilder::new(20, 20)
        .with(10, 12, TerrainProps::new())
        .build();

    let mut objects_fixture: Vec<ObjectState> = Vec::new();

    let lemming_fixture = floater_at_tick(10, LemmingAnimation::Umbrella, 0);
    let mut lemming = lemming_fixture.clone();

    let verdict = lemming.tick(&mut terrain_fixture, &mut objects_fixture);

    assert_eq!(verdict, LemmingVerdict::Continue);
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
