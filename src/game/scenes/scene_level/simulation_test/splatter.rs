use crate::{
    scenes::scene_level::simulation::{LemmingVerdict, test::fixture::TerrainFixtureBuilder},
    state::{Activity, Direction, LemmingAnimation, LemmingState, ObjectState},
};

#[test]
fn splatting_advances_animation() {
    let terrain_fixture = TerrainFixtureBuilder::new(20, 20).build();

    let mut objects_fixture: Vec<ObjectState> = Vec::new();

    let lemming_fixture = LemmingState::fixture(10, 10, Direction::Right, Activity::Splatting);
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
fn splatting_removes_lemming() {
    let terrain_fixture = TerrainFixtureBuilder::new(20, 20).build();

    let mut objects_fixture: Vec<ObjectState> = Vec::new();

    let lemming_fixture = LemmingState {
        frame: LemmingAnimation::Splatting.frame_count() - 1,
        ..LemmingState::fixture(10, 10, Direction::Right, Activity::Splatting)
    };
    let mut lemming = lemming_fixture.clone();

    let verdict = lemming.tick(&terrain_fixture, &mut objects_fixture);

    assert_eq!(verdict, LemmingVerdict::Death);
}
