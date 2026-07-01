use crate::{
    scenes::scene_level::{
        simulation::{DIG_LINE_WIDTH, DIG_X_OFFSET, test::fixture::TerrainFixtureBuilder},
        terrain_diff::{TerrainDiff, TerrainDiffKind},
    },
    state::{
        Activity, ActivityStateDigging, Direction, LemmingAnimation, LemmingState, ObjectState,
        TerrainProps,
    },
};

#[test]
fn digger_digs_three_lines_and_moves_down_on_first_frame() {
    let mut terrain_fixture = TerrainFixtureBuilder::new(20, 20)
        .with_block(5, 5, 10, 10, TerrainProps::new())
        .build();

    let mut objects_fixture: Vec<ObjectState> = Vec::new();

    let lemming_fixture =
        LemmingState::fixture(8, 7, Direction::Left, Activity::Digging(Default::default()))
            .with_frame(15);
    let mut lemming = lemming_fixture.clone();

    lemming.tick(&mut terrain_fixture, &mut objects_fixture);

    assert_eq!(
        lemming,
        LemmingState {
            y: 8,
            frame: 0,
            activity: Activity::Digging(ActivityStateDigging { newborn: false }),
            ..lemming_fixture
        }
    );

    assert!(terrain_fixture.is_block_blank(8 + DIG_X_OFFSET, 5, DIG_LINE_WIDTH, 3));
    assert_eq!(
        terrain_fixture.sorted_diff(),
        &[
            TerrainDiff {
                kind: TerrainDiffKind::Dig,
                x: 8 + DIG_X_OFFSET,
                y: 5
            },
            TerrainDiff {
                kind: TerrainDiffKind::Dig,
                x: 8 + DIG_X_OFFSET,
                y: 6
            },
            TerrainDiff {
                kind: TerrainDiffKind::Dig,
                x: 8 + DIG_X_OFFSET,
                y: 7
            }
        ]
    );
}

#[test]
fn digger_digs_and_moves_down_on_frames_15_and_7() {
    for frame in 0..LemmingAnimation::Digging.frame_count() {
        let mut terrain_fixture = TerrainFixtureBuilder::new(20, 20)
            .with_block(5, 5, 10, 10, TerrainProps::new())
            .build();

        let mut objects_fixture: Vec<ObjectState> = Vec::new();

        let lemming_fixture = LemmingState::fixture(
            8,
            5,
            Direction::Left,
            Activity::Digging(ActivityStateDigging { newborn: false }),
        )
        .with_frame(frame);
        let mut lemming = lemming_fixture.clone();

        lemming.tick(&mut terrain_fixture, &mut objects_fixture);

        match frame {
            7 | 15 => {
                assert_eq!(
                    lemming,
                    LemmingState {
                        y: 6,
                        frame: (frame + 1) % LemmingAnimation::Digging.frame_count(),
                        ..lemming_fixture
                    }
                );

                assert!(terrain_fixture.is_row_blank(8 + DIG_X_OFFSET, 5, DIG_LINE_WIDTH));
                assert_eq!(
                    terrain_fixture.sorted_diff(),
                    &[TerrainDiff {
                        kind: TerrainDiffKind::Dig,
                        x: 8 + DIG_X_OFFSET,
                        y: 5
                    },]
                );
            }
            _ => {
                assert_eq!(
                    lemming,
                    LemmingState {
                        frame: (frame + 1) % LemmingAnimation::Digging.frame_count(),
                        ..lemming_fixture
                    }
                );

                assert!(!terrain_fixture.is_row_blank(8 + DIG_X_OFFSET, 5, DIG_LINE_WIDTH));
                assert_eq!(terrain_fixture.sorted_diff(), &[]);
            }
        }
    }
}

#[test]
fn digger_transitions_to_faller_if_it_cannot_dig() {
    let mut terrain_fixture = TerrainFixtureBuilder::new(20, 20).build();

    let mut objects_fixture: Vec<ObjectState> = Vec::new();

    let lemming_fixture = LemmingState::fixture(
        8,
        5,
        Direction::Left,
        Activity::Digging(ActivityStateDigging { newborn: false }),
    )
    .with_frame(15);
    let mut lemming = lemming_fixture.clone();

    lemming.tick(&mut terrain_fixture, &mut objects_fixture);

    assert_eq!(
        lemming,
        LemmingState {
            y: 6,
            activity: Activity::Falling(Default::default()),
            animation: LemmingAnimation::Falling,
            frame: 0,
            ..lemming_fixture
        }
    );
}

#[test]
fn digger_does_not_transition_to_faller_if_it_does_not_attempt_to_dig() {
    let mut terrain_fixture = TerrainFixtureBuilder::new(20, 20).build();

    let mut objects_fixture: Vec<ObjectState> = Vec::new();

    let lemming_fixture = LemmingState::fixture(
        8,
        5,
        Direction::Left,
        Activity::Digging(ActivityStateDigging { newborn: false }),
    )
    .with_frame(0);
    let mut lemming = lemming_fixture.clone();

    lemming.tick(&mut terrain_fixture, &mut objects_fixture);

    assert_eq!(
        lemming,
        LemmingState {
            frame: 1,
            ..lemming_fixture
        }
    );
}
