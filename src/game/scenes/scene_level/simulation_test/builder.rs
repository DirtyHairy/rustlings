use crate::{
    scenes::scene_level::{
        simulation::test::fixture::TerrainFixtureBuilder,
        terrain_diff::{TerrainDiff, TerrainDiffKind},
    },
    state::{
        Activity, ActivityStateBuilding, Direction, LemmingAnimation, LemmingState, ObjectsState,
        TerrainProps,
    },
};

#[test]
fn builder_lays_a_brick_in_frame_8() {
    let mut terrain_fixture = TerrainFixtureBuilder::new(20, 20).build();

    let mut objects_fixture: ObjectsState = Vec::new();

    let lemming_fixture = LemmingState {
        frame: 8,
        ..LemmingState::fixture(
            5,
            10,
            Direction::Right,
            Activity::Building(ActivityStateBuilding { bricks: 12 }),
        )
    };
    let mut lemming = lemming_fixture.clone();

    lemming.tick(&mut terrain_fixture, &mut objects_fixture);

    assert_eq!(
        lemming,
        LemmingState {
            frame: 9,
            ignore_blocker: true,
            ..lemming_fixture
        }
    );

    assert_eq!(
        terrain_fixture.sorted_diff(),
        [TerrainDiff {
            kind: TerrainDiffKind::Brick,
            x: 5,
            y: 9
        }]
    );
}

#[test]
fn builder_left_lays_a_brick_at_shifted_x() {
    let mut terrain_fixture = TerrainFixtureBuilder::new(20, 20).build();

    let mut objects_fixture: ObjectsState = Vec::new();

    let lemming_fixture = LemmingState {
        frame: 8,
        ..LemmingState::fixture(
            5,
            10,
            Direction::Left,
            Activity::Building(ActivityStateBuilding { bricks: 12 }),
        )
    };
    let mut lemming = lemming_fixture.clone();

    lemming.tick(&mut terrain_fixture, &mut objects_fixture);

    assert_eq!(
        lemming,
        LemmingState {
            frame: 9,
            ignore_blocker: true,
            ..lemming_fixture
        }
    );

    assert_eq!(
        terrain_fixture.sorted_diff(),
        [TerrainDiff {
            kind: TerrainDiffKind::Brick,
            x: 1,
            y: 9
        }]
    );
}

#[test]
fn laying_a_brick_creates_terrain() {
    let mut terrain_fixture = TerrainFixtureBuilder::new(20, 20).build();

    let mut objects_fixture: ObjectsState = Vec::new();

    let lemming_fixture = LemmingState {
        frame: 8,
        ..LemmingState::fixture(
            5,
            10,
            Direction::Right,
            Activity::Building(ActivityStateBuilding { bricks: 12 }),
        )
    };
    let mut lemming = lemming_fixture.clone();

    lemming.tick(&mut terrain_fixture, &mut objects_fixture);

    for x in 5..11 {
        assert!(terrain_fixture.is_solid(x, 9));
    }
}

#[test]
fn builder_lays_a_brick_in_frame_9_if_there_are_9_bricks_left() {
    let mut terrain_fixture = TerrainFixtureBuilder::new(20, 20).build();

    let mut objects_fixture: ObjectsState = Vec::new();

    let lemming_fixture = LemmingState {
        frame: 9,
        ..LemmingState::fixture(
            5,
            10,
            Direction::Right,
            Activity::Building(ActivityStateBuilding { bricks: 9 }),
        )
    };
    let mut lemming = lemming_fixture.clone();

    lemming.tick(&mut terrain_fixture, &mut objects_fixture);

    assert_eq!(
        lemming,
        LemmingState {
            frame: 10,
            ignore_blocker: true,
            ..lemming_fixture
        }
    );

    assert_eq!(
        terrain_fixture.sorted_diff(),
        [TerrainDiff {
            kind: TerrainDiffKind::Brick,
            x: 5,
            y: 9
        }]
    );
}

#[test]
fn builder_does_not_lay_a_brick_in_frame_9_else() {
    let mut terrain_fixture = TerrainFixtureBuilder::new(20, 20).build();

    let mut objects_fixture: ObjectsState = Vec::new();

    let lemming_fixture = LemmingState {
        frame: 9,
        ..LemmingState::fixture(
            5,
            10,
            Direction::Right,
            Activity::Building(ActivityStateBuilding { bricks: 10 }),
        )
    };
    let mut lemming = lemming_fixture.clone();

    lemming.tick(&mut terrain_fixture, &mut objects_fixture);

    assert_eq!(
        lemming,
        LemmingState {
            frame: 10,
            ..lemming_fixture
        }
    );

    assert_eq!(terrain_fixture.sorted_diff(), []);
}

#[test]
fn builder_moves_up_and_consumes_brick_in_frame_15() {
    let mut terrain_fixture = TerrainFixtureBuilder::new(20, 20).build();

    let mut objects_fixture: ObjectsState = Vec::new();

    let lemming_fixture = LemmingState {
        frame: 15,
        ..LemmingState::fixture(
            5,
            10,
            Direction::Right,
            Activity::Building(ActivityStateBuilding { bricks: 12 }),
        )
    };
    let mut lemming = lemming_fixture.clone();

    lemming.tick(&mut terrain_fixture, &mut objects_fixture);

    assert_eq!(
        lemming,
        LemmingState {
            frame: 0,
            x: 7,
            y: 9,
            activity: Activity::Building(ActivityStateBuilding { bricks: 11 }),
            ..lemming_fixture
        }
    );

    assert_eq!(terrain_fixture.sorted_diff(), []);
}

#[test]
fn builder_turns_and_walks_if_hitting_the_edge_in_first_step_in_frame_15() {
    let mut terrain_fixture = TerrainFixtureBuilder::new(20, 20).build();

    let mut objects_fixture: ObjectsState = Vec::new();

    let lemming_fixture = LemmingState {
        frame: 15,
        ..LemmingState::fixture(
            19,
            10,
            Direction::Right,
            Activity::Building(ActivityStateBuilding { bricks: 1 }),
        )
    };
    let mut lemming = lemming_fixture.clone();

    lemming.tick(&mut terrain_fixture, &mut objects_fixture);

    assert_eq!(
        lemming,
        LemmingState {
            frame: 0,
            x: 20,
            y: 9,
            activity: Activity::Walking,
            animation: Some(LemmingAnimation::Walking),
            direction: Direction::Left,
            ..lemming_fixture
        }
    );

    assert_eq!(terrain_fixture.sorted_diff(), []);
}

#[test]
fn builder_turns_and_walks_after_hitting_an_obstacle_in_first_step_in_frame_15() {
    let mut terrain_fixture = TerrainFixtureBuilder::new(20, 20)
        .with(11, 8, TerrainProps::new().with_solid(true))
        .build();

    let mut objects_fixture: ObjectsState = Vec::new();

    let lemming_fixture = LemmingState {
        frame: 15,
        ..LemmingState::fixture(
            10,
            10,
            Direction::Right,
            Activity::Building(ActivityStateBuilding { bricks: 1 }),
        )
    };
    let mut lemming = lemming_fixture.clone();

    lemming.tick(&mut terrain_fixture, &mut objects_fixture);

    assert_eq!(
        lemming,
        LemmingState {
            frame: 0,
            x: 11,
            y: 9,
            activity: Activity::Walking,
            animation: Some(LemmingAnimation::Walking),
            direction: Direction::Left,
            ..lemming_fixture
        }
    );

    assert_eq!(terrain_fixture.sorted_diff(), []);
}

#[test]
fn aborting_builder_turns_again_after_hitting_the_ceiling_in_first_step_in_frame_15() {
    let mut terrain_fixture = TerrainFixtureBuilder::new(20, 20).build();

    let mut objects_fixture: ObjectsState = Vec::new();

    let lemming_fixture = LemmingState {
        frame: 15,
        ..LemmingState::fixture(
            19,
            5,
            Direction::Right,
            Activity::Building(ActivityStateBuilding { bricks: 1 }),
        )
    };
    let mut lemming = lemming_fixture.clone();

    lemming.tick(&mut terrain_fixture, &mut objects_fixture);

    assert_eq!(
        lemming,
        LemmingState {
            frame: 0,
            x: 20,
            y: 3,
            activity: Activity::Walking,
            animation: Some(LemmingAnimation::Walking),
            ..lemming_fixture
        }
    );

    assert_eq!(terrain_fixture.sorted_diff(), []);
}

#[test]
fn builder_turns_and_walks_after_hitting_an_obstacle_in_second_step_in_frame_15() {
    let mut terrain_fixture = TerrainFixtureBuilder::new(20, 20)
        .with(12, 8, TerrainProps::new().with_solid(true))
        .build();

    let mut objects_fixture: ObjectsState = Vec::new();

    let lemming_fixture = LemmingState {
        frame: 15,
        ..LemmingState::fixture(
            10,
            10,
            Direction::Right,
            Activity::Building(ActivityStateBuilding { bricks: 1 }),
        )
    };
    let mut lemming = lemming_fixture.clone();

    lemming.tick(&mut terrain_fixture, &mut objects_fixture);

    assert_eq!(
        lemming,
        LemmingState {
            frame: 0,
            x: 12,
            y: 9,
            activity: Activity::Walking,
            animation: Some(LemmingAnimation::Walking),
            direction: Direction::Left,
            ..lemming_fixture
        }
    );

    assert_eq!(terrain_fixture.sorted_diff(), []);
}

#[test]
fn aborting_builder_turns_again_after_hitting_the_ceiling_in_second_step_in_frame_15() {
    let mut terrain_fixture = TerrainFixtureBuilder::new(20, 20)
        .with(12, 3, TerrainProps::new().with_solid(true))
        .build();

    let mut objects_fixture: ObjectsState = Vec::new();

    let lemming_fixture = LemmingState {
        frame: 15,
        ..LemmingState::fixture(
            10,
            5,
            Direction::Right,
            Activity::Building(ActivityStateBuilding { bricks: 1 }),
        )
    };
    let mut lemming = lemming_fixture.clone();

    lemming.tick(&mut terrain_fixture, &mut objects_fixture);

    assert_eq!(
        lemming,
        LemmingState {
            frame: 0,
            x: 12,
            y: 3,
            activity: Activity::Walking,
            animation: Some(LemmingAnimation::Walking),
            ..lemming_fixture
        }
    );

    assert_eq!(terrain_fixture.sorted_diff(), []);
}

#[test]
fn builder_transforms_to_shrugger_if_bricks_run_out_in_frame_15() {
    let mut terrain_fixture = TerrainFixtureBuilder::new(20, 20).build();

    let mut objects_fixture: ObjectsState = Vec::new();

    let lemming_fixture = LemmingState {
        frame: 15,
        ..LemmingState::fixture(
            10,
            10,
            Direction::Right,
            Activity::Building(ActivityStateBuilding { bricks: 1 }),
        )
    };
    let mut lemming = lemming_fixture.clone();

    lemming.tick(&mut terrain_fixture, &mut objects_fixture);

    assert_eq!(
        lemming,
        LemmingState {
            frame: 0,
            x: 12,
            y: 9,
            activity: Activity::Shrugging,
            animation: Some(LemmingAnimation::Shrugging),
            ..lemming_fixture
        }
    );

    assert_eq!(terrain_fixture.sorted_diff(), []);
}

#[test]
fn builder_transforming_to_shrugger_turns_if_it_hits_the_ceiling_in_frame_15() {
    let mut terrain_fixture = TerrainFixtureBuilder::new(20, 20).build();

    let mut objects_fixture: ObjectsState = Vec::new();

    let lemming_fixture = LemmingState {
        frame: 15,
        ..LemmingState::fixture(
            10,
            5,
            Direction::Right,
            Activity::Building(ActivityStateBuilding { bricks: 1 }),
        )
    };
    let mut lemming = lemming_fixture.clone();

    lemming.tick(&mut terrain_fixture, &mut objects_fixture);

    assert_eq!(
        lemming,
        LemmingState {
            frame: 0,
            x: 12,
            y: 3,
            direction: Direction::Left,
            activity: Activity::Shrugging,
            animation: Some(LemmingAnimation::Shrugging),
            ..lemming_fixture
        }
    );

    assert_eq!(terrain_fixture.sorted_diff(), []);
}

#[test]
fn builder_turns_and_walks_if_hitting_the_boundary_in_frame_15() {
    let mut terrain_fixture = TerrainFixtureBuilder::new(20, 20).build();

    let mut objects_fixture: ObjectsState = Vec::new();

    let lemming_fixture = LemmingState {
        frame: 15,
        ..LemmingState::fixture(
            17,
            10,
            Direction::Right,
            Activity::Building(ActivityStateBuilding { bricks: 2 }),
        )
    };
    let mut lemming = lemming_fixture.clone();

    lemming.tick(&mut terrain_fixture, &mut objects_fixture);

    assert_eq!(
        lemming,
        LemmingState {
            frame: 0,
            x: 19,
            y: 9,
            direction: Direction::Left,
            activity: Activity::Walking,
            animation: Some(LemmingAnimation::Walking),
            ..lemming_fixture
        }
    );

    assert_eq!(terrain_fixture.sorted_diff(), []);
}

#[test]
fn builder_turns_and_walks_if_seeing_an_obstacle_ahead_in_frame_15() {
    let mut terrain_fixture = TerrainFixtureBuilder::new(20, 20)
        .with(14, 9, TerrainProps::new().with_solid(true))
        .build();

    let mut objects_fixture: ObjectsState = Vec::new();

    let lemming_fixture = LemmingState {
        frame: 15,
        ..LemmingState::fixture(
            10,
            19,
            Direction::Right,
            Activity::Building(ActivityStateBuilding { bricks: 2 }),
        )
    };
    let mut lemming = lemming_fixture.clone();

    lemming.tick(&mut terrain_fixture, &mut objects_fixture);

    assert_eq!(
        lemming,
        LemmingState {
            frame: 0,
            x: 12,
            y: 18,
            direction: Direction::Left,
            activity: Activity::Walking,
            animation: Some(LemmingAnimation::Walking),
            ..lemming_fixture
        }
    );

    assert_eq!(terrain_fixture.sorted_diff(), []);
}

#[test]
fn builder_turned_shrugger_does_not_care_about_the_boundary_in_frame_15() {
    let mut terrain_fixture = TerrainFixtureBuilder::new(20, 20).build();

    let mut objects_fixture: ObjectsState = Vec::new();

    let lemming_fixture = LemmingState {
        frame: 15,
        ..LemmingState::fixture(
            17,
            10,
            Direction::Right,
            Activity::Building(ActivityStateBuilding { bricks: 1 }),
        )
    };
    let mut lemming = lemming_fixture.clone();

    lemming.tick(&mut terrain_fixture, &mut objects_fixture);

    assert_eq!(
        lemming,
        LemmingState {
            frame: 0,
            x: 19,
            y: 9,
            activity: Activity::Shrugging,
            animation: Some(LemmingAnimation::Shrugging),
            ..lemming_fixture
        }
    );

    assert_eq!(terrain_fixture.sorted_diff(), []);
}

#[test]
fn builder_turns_twice_and_walks_if_hitting_the_boundary_and_the_ceiling_in_frame_15() {
    let mut terrain_fixture = TerrainFixtureBuilder::new(20, 20).build();

    let mut objects_fixture: ObjectsState = Vec::new();

    let lemming_fixture = LemmingState {
        frame: 15,
        ..LemmingState::fixture(
            17,
            5,
            Direction::Right,
            Activity::Building(ActivityStateBuilding { bricks: 2 }),
        )
    };
    let mut lemming = lemming_fixture.clone();

    lemming.tick(&mut terrain_fixture, &mut objects_fixture);

    assert_eq!(
        lemming,
        LemmingState {
            frame: 0,
            x: 19,
            y: 3,
            activity: Activity::Walking,
            animation: Some(LemmingAnimation::Walking),
            ..lemming_fixture
        }
    );

    assert_eq!(terrain_fixture.sorted_diff(), []);
}

#[test]
fn builder_walks_without_turning_after_hitting_the_ceiling_in_frame_15() {
    let mut terrain_fixture = TerrainFixtureBuilder::new(20, 20).build();

    let mut objects_fixture: ObjectsState = Vec::new();

    let lemming_fixture = LemmingState {
        frame: 15,
        ..LemmingState::fixture(
            10,
            8,
            Direction::Right,
            Activity::Building(ActivityStateBuilding { bricks: 2 }),
        )
    };
    let mut lemming = lemming_fixture.clone();

    lemming.tick(&mut terrain_fixture, &mut objects_fixture);

    assert_eq!(
        lemming,
        LemmingState {
            frame: 0,
            x: 12,
            y: 7,
            activity: Activity::Walking,
            animation: Some(LemmingAnimation::Walking),
            ..lemming_fixture
        }
    );

    assert_eq!(terrain_fixture.sorted_diff(), []);
}

#[test]
fn builder_walks_and_turns_after_hitting_the_ceiling_close_in_frame_15() {
    let mut terrain_fixture = TerrainFixtureBuilder::new(20, 20).build();

    let mut objects_fixture: ObjectsState = Vec::new();

    let lemming_fixture = LemmingState {
        frame: 15,
        ..LemmingState::fixture(
            10,
            5,
            Direction::Right,
            Activity::Building(ActivityStateBuilding { bricks: 2 }),
        )
    };
    let mut lemming = lemming_fixture.clone();

    lemming.tick(&mut terrain_fixture, &mut objects_fixture);

    assert_eq!(
        lemming,
        LemmingState {
            frame: 0,
            x: 12,
            y: 3,
            direction: Direction::Left,
            activity: Activity::Walking,
            animation: Some(LemmingAnimation::Walking),
            ..lemming_fixture
        }
    );

    assert_eq!(terrain_fixture.sorted_diff(), []);
}

#[test]
fn builder_does_nothing_on_all_other_frames() {
    for frame in (0..=7).chain(10..=14) {
        let mut terrain_fixture = TerrainFixtureBuilder::new(20, 20).build();

        let mut objects_fixture: ObjectsState = Vec::new();

        let lemming_fixture = LemmingState {
            frame,
            ..LemmingState::fixture(
                10,
                10,
                Direction::Right,
                Activity::Building(ActivityStateBuilding { bricks: 2 }),
            )
        };
        let mut lemming = lemming_fixture.clone();

        lemming.tick(&mut terrain_fixture, &mut objects_fixture);

        assert_eq!(
            lemming,
            LemmingState {
                frame: frame + 1,
                x: 10,
                y: 10,
                ..lemming_fixture
            }
        );
    }
}

#[test]
fn shrugger_does_nothing_on_frames_0_to_6() {
    for frame in 0..=6 {
        let mut terrain_fixture = TerrainFixtureBuilder::new(20, 20).build();

        let mut objects_fixture: ObjectsState = Vec::new();

        let lemming_fixture = LemmingState {
            frame,
            ..LemmingState::fixture(10, 10, Direction::Right, Activity::Shrugging)
        };
        let mut lemming = lemming_fixture.clone();

        lemming.tick(&mut terrain_fixture, &mut objects_fixture);

        assert_eq!(
            lemming,
            LemmingState {
                frame: frame + 1,
                x: 10,
                y: 10,
                ..lemming_fixture
            }
        );
    }
}

#[test]
fn shrugger_transitions_to_walker_on_frame_7() {
    let mut terrain_fixture = TerrainFixtureBuilder::new(20, 20).build();

    let mut objects_fixture: ObjectsState = Vec::new();

    let lemming_fixture = LemmingState {
        frame: 7,
        ..LemmingState::fixture(10, 10, Direction::Right, Activity::Shrugging)
    };
    let mut lemming = lemming_fixture.clone();

    lemming.tick(&mut terrain_fixture, &mut objects_fixture);

    assert_eq!(
        lemming,
        LemmingState {
            frame: 0,
            x: 10,
            y: 10,
            activity: Activity::Walking,
            animation: Some(LemmingAnimation::Walking),
            ..lemming_fixture
        }
    );
}
