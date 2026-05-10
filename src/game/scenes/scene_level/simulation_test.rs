use crate::{
    scenes::scene_level::simulation::*,
    state::{ActivityStateFalling, Direction},
};

struct TerrainFixtureBuilder {
    width: u32,
    height: u32,
    map: Vec<TerrainProps>,
}

impl TerrainFixtureBuilder {
    fn new(width: u32, height: u32) -> Self {
        let map: Vec<TerrainProps> = vec![Default::default(); (width * height) as usize];

        Self { width, height, map }
    }

    fn with(mut self, x: u32, y: u32, terrain_props: TerrainProps) -> Self {
        self.map[(x + y * self.width) as usize] = terrain_props.with_solid(true);

        self
    }

    fn with_col(mut self, x: u32, y: u32, len: u32, terrain_props: TerrainProps) -> Self {
        for row in 0..len {
            self.map[(x + y.saturating_sub(row) * self.width) as usize] =
                terrain_props.with_solid(true);
        }

        self
    }

    fn with_row(mut self, x: u32, y: u32, len: u32, terrain_props: TerrainProps) -> Self {
        for col in 0..len {
            self.map[(x + col + y * self.width) as usize] = terrain_props.with_solid(true);
        }

        self
    }

    fn build(self) -> TerrainMap<'static> {
        TerrainMap::new(self.width, self.height, self.map.leak())
    }
}

impl LemmingState {
    fn fixture(x: i32, y: i32, direction: Direction, activity: Activity) -> Self {
        let mut lemming = LemmingState {
            x,
            y,
            direction,
            ..Default::default()
        };

        lemming.transition_to(activity);

        lemming
    }
}

#[test]
fn walker_steps_up_right() {
    for step in 0..=MAX_STEP_UP {
        let terrain_fixture = TerrainFixtureBuilder::new(20, 20)
            .with(10, 19, TerrainProps::new())
            .with_col(11, 19, step + 1, TerrainProps::new())
            .build();

        let mut objects_fixture: Vec<ObjectState> = Vec::new();

        let lemming_fixture = LemmingState::fixture(10, 19, Direction::Right, Activity::Walking);
        let mut lemming = lemming_fixture.clone();

        lemming.tick(&terrain_fixture, &mut objects_fixture);

        assert_eq!(
            lemming,
            LemmingState {
                x: 11,
                y: 19 - step as i32,
                frame: 1,
                ..lemming_fixture
            },
            "should step {} pixels",
            step
        );
    }
}

#[test]
fn walker_steps_up_left() {
    for step in 0..=MAX_STEP_UP {
        let terrain_fixture = TerrainFixtureBuilder::new(20, 20)
            .with(10, 19, TerrainProps::new())
            .with_col(9, 19, step + 1, TerrainProps::new())
            .build();

        let mut objects_fixture: Vec<ObjectState> = Vec::new();

        let lemming_fixture = LemmingState::fixture(10, 19, Direction::Left, Activity::Walking);
        let mut lemming = lemming_fixture.clone();

        lemming.tick(&terrain_fixture, &mut objects_fixture);

        assert_eq!(
            lemming,
            LemmingState {
                x: 9,
                y: 19 - step as i32,
                frame: 1,
                ..lemming_fixture
            },
            "should step {} pixels",
            step
        );
    }
}

#[test]
fn walker_jumps_right() {
    for step in MAX_STEP_UP + 1..=MAX_JUMP {
        let terrain_fixture = TerrainFixtureBuilder::new(20, 20)
            .with(10, 19, TerrainProps::new())
            .with_col(11, 19, step + 1, TerrainProps::new())
            .build();

        let mut objects_fixture: Vec<ObjectState> = Vec::new();

        let lemming_fixture = LemmingState::fixture(10, 19, Direction::Right, Activity::Walking);
        let mut lemming = lemming_fixture.clone();

        lemming.tick(&terrain_fixture, &mut objects_fixture);

        assert_eq!(
            lemming,
            LemmingState {
                x: 11,
                y: 17,
                frame: 0,
                activity: Activity::Jumping,
                animation: LemmingAnimation::Jumping,
                ..lemming_fixture
            },
            "should jump {} pixels",
            step
        );
    }
}

#[test]
fn walker_jumps_left() {
    for step in MAX_STEP_UP + 1..=MAX_JUMP {
        let terrain_fixture = TerrainFixtureBuilder::new(20, 20)
            .with(10, 19, TerrainProps::new())
            .with_col(9, 19, step + 1, TerrainProps::new())
            .build();

        let mut objects_fixture: Vec<ObjectState> = Vec::new();

        let lemming_fixture = LemmingState::fixture(10, 19, Direction::Left, Activity::Walking);
        let mut lemming = lemming_fixture.clone();

        lemming.tick(&terrain_fixture, &mut objects_fixture);

        assert_eq!(
            lemming,
            LemmingState {
                x: 9,
                y: 17,
                frame: 0,
                activity: Activity::Jumping,
                animation: LemmingAnimation::Jumping,
                ..lemming_fixture
            },
            "should jump {} pixels",
            step
        );
    }
}

#[test]
fn walker_should_turn_left() {
    let terrain_fixture = TerrainFixtureBuilder::new(20, 20)
        .with(10, 19, TerrainProps::new())
        .with_col(11, 19, MAX_JUMP + 2, TerrainProps::new())
        .build();

    let mut objects_fixture: Vec<ObjectState> = Vec::new();

    let lemming_fixture = LemmingState::fixture(10, 19, Direction::Right, Activity::Walking);
    let mut lemming = lemming_fixture.clone();

    lemming.tick(&terrain_fixture, &mut objects_fixture);

    assert_eq!(
        lemming,
        LemmingState {
            x: 11,
            frame: 1,
            direction: Direction::Left,
            ..lemming_fixture
        }
    );
}

#[test]
fn walker_should_turn_right() {
    let terrain_fixture = TerrainFixtureBuilder::new(20, 20)
        .with(10, 19, TerrainProps::new())
        .with_col(9, 19, MAX_JUMP + 2, TerrainProps::new())
        .build();

    let mut objects_fixture: Vec<ObjectState> = Vec::new();

    let lemming_fixture = LemmingState::fixture(10, 19, Direction::Left, Activity::Walking);
    let mut lemming = lemming_fixture.clone();

    lemming.tick(&terrain_fixture, &mut objects_fixture);

    assert_eq!(
        lemming,
        LemmingState {
            x: 9,
            frame: 1,
            direction: Direction::Right,
            ..lemming_fixture
        }
    );
}

#[test]
fn walker_wraps_animation() {
    let terrain_fixture = TerrainFixtureBuilder::new(20, 20)
        .with_row(10, 19, 2, TerrainProps::new())
        .build();

    let mut objects_fixture: Vec<ObjectState> = Vec::new();

    let lemming_fixture = LemmingState {
        frame: LemmingAnimation::Walking.frame_count() - 1,
        ..LemmingState::fixture(10, 19, Direction::Right, Activity::Walking)
    };
    let mut lemming = lemming_fixture.clone();

    lemming.tick(&terrain_fixture, &mut objects_fixture);

    assert_eq!(
        lemming,
        LemmingState {
            x: 11,
            frame: 0,
            ..lemming_fixture
        }
    );
}

#[test]
fn walker_steps_down_right() {
    for step in 1..=MAX_STEP_DOWN {
        let terrain_fixture = TerrainFixtureBuilder::new(20, 20)
            .with(10, 10, TerrainProps::new())
            .with(11, 10 + step, TerrainProps::new())
            .build();

        let mut objects_fixture: Vec<ObjectState> = Vec::new();

        let lemming_fixture = LemmingState::fixture(10, 10, Direction::Right, Activity::Walking);
        let mut lemming = lemming_fixture.clone();

        lemming.tick(&terrain_fixture, &mut objects_fixture);

        assert_eq!(
            lemming,
            LemmingState {
                x: 11,
                y: (10 + step) as i32,
                frame: 1,
                ..lemming_fixture
            },
            "should step down {} pixels",
            step
        );
    }
}

#[test]
fn walker_steps_down_left() {
    for step in 1..=MAX_STEP_DOWN {
        let terrain_fixture = TerrainFixtureBuilder::new(20, 20)
            .with(10, 10, TerrainProps::new())
            .with(9, 10 + step, TerrainProps::new())
            .build();

        let mut objects_fixture: Vec<ObjectState> = Vec::new();

        let lemming_fixture = LemmingState::fixture(10, 10, Direction::Left, Activity::Walking);
        let mut lemming = lemming_fixture.clone();

        lemming.tick(&terrain_fixture, &mut objects_fixture);

        assert_eq!(
            lemming,
            LemmingState {
                x: 9,
                y: (10 + step) as i32,
                frame: 1,
                ..lemming_fixture
            },
            "should step down {} pixels",
            step
        );
    }
}

#[test]
fn walker_starts_falling_right() {
    let terrain_fixture = TerrainFixtureBuilder::new(20, 20)
        .with(10, 10, TerrainProps::new())
        .with(11, 10 + MAX_STEP_DOWN + 1, TerrainProps::new())
        .build();

    let mut objects_fixture: Vec<ObjectState> = Vec::new();

    let lemming_fixture = LemmingState::fixture(10, 10, Direction::Right, Activity::Walking);
    let mut lemming = lemming_fixture.clone();

    lemming.tick(&terrain_fixture, &mut objects_fixture);

    assert_eq!(
        lemming,
        LemmingState {
            x: 11,
            y: 10 + (MAX_STEP_DOWN + 1) as i32,
            frame: 0,
            activity: Activity::Falling(Default::default()),
            animation: LemmingAnimation::Falling,
            ..lemming_fixture
        }
    );
}

#[test]
fn walker_starts_falling_left() {
    let terrain_fixture = TerrainFixtureBuilder::new(20, 20)
        .with(10, 10, TerrainProps::new())
        .with(9, 10 + MAX_STEP_DOWN + 1, TerrainProps::new())
        .build();

    let mut objects_fixture: Vec<ObjectState> = Vec::new();

    let lemming_fixture = LemmingState::fixture(10, 10, Direction::Left, Activity::Walking);
    let mut lemming = lemming_fixture.clone();

    lemming.tick(&terrain_fixture, &mut objects_fixture);

    assert_eq!(
        lemming,
        LemmingState {
            x: 9,
            y: 10 + (MAX_STEP_DOWN + 1) as i32,
            frame: 0,
            activity: Activity::Falling(Default::default()),
            animation: LemmingAnimation::Falling,
            ..lemming_fixture
        }
    );
}

#[test]
fn walker_turns_at_left_boundary() {
    let terrain_fixture = TerrainFixtureBuilder::new(20, 20)
        .with_row(0, 10, 2, TerrainProps::new())
        .build();

    let mut objects_fixture: Vec<ObjectState> = Vec::new();

    let lemming_fixture = LemmingState::fixture(1, 10, Direction::Left, Activity::Walking);
    let mut lemming = lemming_fixture.clone();

    lemming.tick(&terrain_fixture, &mut objects_fixture);

    assert_eq!(
        lemming,
        LemmingState {
            x: 0,
            frame: 1,
            direction: Direction::Right,
            ..lemming_fixture
        }
    );
}

#[test]
fn walker_turns_at_right_boundary() {
    let terrain_fixture = TerrainFixtureBuilder::new(20, 20)
        .with_row(18, 10, 2, TerrainProps::new())
        .build();

    let mut objects_fixture: Vec<ObjectState> = Vec::new();

    let lemming_fixture = LemmingState::fixture(18, 10, Direction::Right, Activity::Walking);
    let mut lemming = lemming_fixture.clone();

    lemming.tick(&terrain_fixture, &mut objects_fixture);

    assert_eq!(
        lemming,
        LemmingState {
            x: 19,
            frame: 1,
            direction: Direction::Left,
            ..lemming_fixture
        }
    );
}

#[test]
fn walker_hits_ceiling_right() {
    let terrain_fixture = TerrainFixtureBuilder::new(20, 20)
        .with(10, MIN_FOOT_Y as u32, TerrainProps::new())
        .with_col(11, MIN_FOOT_Y as u32, 2, TerrainProps::new())
        .build();

    let mut objects_fixture: Vec<ObjectState> = Vec::new();

    let lemming_fixture =
        LemmingState::fixture(10, MIN_FOOT_Y, Direction::Right, Activity::Walking);
    let mut lemming = lemming_fixture.clone();

    lemming.tick(&terrain_fixture, &mut objects_fixture);

    assert_eq!(
        lemming,
        LemmingState {
            x: 11,
            y: CEILING_HIT_Y_RESET,
            frame: 1,
            direction: Direction::Left,
            ..lemming_fixture
        }
    );
}

#[test]
fn walker_hits_ceiling_left() {
    let terrain_fixture = TerrainFixtureBuilder::new(20, 20)
        .with(10, MIN_FOOT_Y as u32, TerrainProps::new())
        .with_col(9, MIN_FOOT_Y as u32, 2, TerrainProps::new())
        .build();

    let mut objects_fixture: Vec<ObjectState> = Vec::new();

    let lemming_fixture = LemmingState::fixture(10, MIN_FOOT_Y, Direction::Left, Activity::Walking);
    let mut lemming = lemming_fixture.clone();

    lemming.tick(&terrain_fixture, &mut objects_fixture);

    assert_eq!(
        lemming,
        LemmingState {
            x: 9,
            y: CEILING_HIT_Y_RESET,
            frame: 1,
            direction: Direction::Right,
            ..lemming_fixture
        }
    );
}

#[test]
fn faller_falls() {
    let terrain_fixture = TerrainFixtureBuilder::new(20, 20).build();

    let mut objects_fixture: Vec<ObjectState> = Vec::new();

    let lemming_fixture = LemmingState::fixture(
        10,
        10,
        Direction::Right,
        Activity::Falling(Default::default()),
    );
    let mut lemming = lemming_fixture.clone();

    lemming.tick(&terrain_fixture, &mut objects_fixture);

    assert_eq!(
        lemming,
        LemmingState {
            y: 10 + FALL_DISTANCE_PER_FRAME as i32,
            frame: 1,
            activity: Activity::Falling(ActivityStateFalling {
                delta_y: FALL_DISTANCE_PER_FRAME
            }),
            ..lemming_fixture
        }
    );
}

#[test]
fn faller_lands_safely() {
    let terrain_fixture = TerrainFixtureBuilder::new(20, 20)
        .with(10, 12, TerrainProps::new())
        .build();

    let mut objects_fixture: Vec<ObjectState> = Vec::new();

    let lemming_fixture = LemmingState::fixture(
        10,
        10,
        Direction::Right,
        Activity::Falling(ActivityStateFalling {
            delta_y: MAX_SAFE_FALL - 2,
        }),
    );
    let mut lemming = lemming_fixture.clone();

    lemming.tick(&terrain_fixture, &mut objects_fixture);

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

#[test]
fn faller_splats() {
    let terrain_fixture = TerrainFixtureBuilder::new(20, 20)
        .with(10, 12, TerrainProps::new())
        .build();

    let mut objects_fixture: Vec<ObjectState> = Vec::new();

    let lemming_fixture = LemmingState::fixture(
        10,
        10,
        Direction::Right,
        Activity::Falling(ActivityStateFalling {
            delta_y: MAX_SAFE_FALL - 1,
        }),
    );
    let mut lemming = lemming_fixture.clone();

    lemming.tick(&terrain_fixture, &mut objects_fixture);

    assert_eq!(
        lemming,
        LemmingState {
            y: 12,
            frame: 0,
            activity: Activity::Splatting,
            animation: LemmingAnimation::Splatting,
            ..lemming_fixture
        }
    );
}

#[test]
fn faller_wraps_animation() {
    let terrain_fixture = TerrainFixtureBuilder::new(20, 20).build();

    let mut objects_fixture: Vec<ObjectState> = Vec::new();

    let lemming_fixture = LemmingState {
        frame: LemmingAnimation::Falling.frame_count() - 1,
        ..LemmingState::fixture(
            10,
            10,
            Direction::Right,
            Activity::Falling(Default::default()),
        )
    };
    let mut lemming = lemming_fixture.clone();

    lemming.tick(&terrain_fixture, &mut objects_fixture);

    assert_eq!(
        lemming,
        LemmingState {
            y: 10 + FALL_DISTANCE_PER_FRAME as i32,
            frame: 0,
            activity: Activity::Falling(ActivityStateFalling {
                delta_y: FALL_DISTANCE_PER_FRAME
            }),
            ..lemming_fixture
        }
    );
}

#[test]
fn jumper_continues() {
    let terrain_fixture = TerrainFixtureBuilder::new(20, 20)
        .with_col(10, 15, 4, TerrainProps::new())
        .build();

    let mut objects_fixture: Vec<ObjectState> = Vec::new();

    let lemming_fixture = LemmingState::fixture(10, 15, Direction::Right, Activity::Jumping);
    let mut lemming = lemming_fixture.clone();

    lemming.tick(&terrain_fixture, &mut objects_fixture);

    assert_eq!(
        lemming,
        LemmingState {
            y: 15 - JUMP_DISTANCE as i32,
            ..lemming_fixture
        }
    );
}

#[test]
fn jumper_continues_exact() {
    let terrain_fixture = TerrainFixtureBuilder::new(20, 20)
        .with_col(10, 15, 3, TerrainProps::new())
        .build();

    let mut objects_fixture: Vec<ObjectState> = Vec::new();

    let lemming_fixture = LemmingState::fixture(10, 15, Direction::Right, Activity::Jumping);
    let mut lemming = lemming_fixture.clone();

    lemming.tick(&terrain_fixture, &mut objects_fixture);

    assert_eq!(
        lemming,
        LemmingState {
            y: 15 - JUMP_DISTANCE as i32,
            ..lemming_fixture
        }
    );
}

#[test]
fn jumper_clears_obstacle() {
    let terrain_fixture = TerrainFixtureBuilder::new(20, 20)
        .with_col(10, 15, 2, TerrainProps::new())
        .build();

    let mut objects_fixture: Vec<ObjectState> = Vec::new();

    let lemming_fixture = LemmingState::fixture(10, 15, Direction::Right, Activity::Jumping);
    let mut lemming = lemming_fixture.clone();

    lemming.tick(&terrain_fixture, &mut objects_fixture);

    assert_eq!(
        lemming,
        LemmingState {
            y: 14,
            frame: 0,
            activity: Activity::Walking,
            animation: LemmingAnimation::Walking,
            ..lemming_fixture
        }
    );
}

#[test]
fn jumper_clears_obstacle_exact() {
    let terrain_fixture = TerrainFixtureBuilder::new(20, 20)
        .with_col(10, 15, 1, TerrainProps::new())
        .build();

    let mut objects_fixture: Vec<ObjectState> = Vec::new();

    let lemming_fixture = LemmingState::fixture(10, 15, Direction::Right, Activity::Jumping);
    let mut lemming = lemming_fixture.clone();

    lemming.tick(&terrain_fixture, &mut objects_fixture);

    assert_eq!(
        lemming,
        LemmingState {
            frame: 0,
            activity: Activity::Walking,
            animation: LemmingAnimation::Walking,
            ..lemming_fixture
        }
    );
}

#[test]
fn jumper_hits_ceiling() {
    let terrain_fixture = TerrainFixtureBuilder::new(20, 20)
        .with_col(10, MIN_FOOT_Y as u32 + 1, 3, TerrainProps::new())
        .build();

    let mut objects_fixture: Vec<ObjectState> = Vec::new();

    let lemming_fixture =
        LemmingState::fixture(10, MIN_FOOT_Y + 1, Direction::Right, Activity::Jumping);
    let mut lemming = lemming_fixture.clone();

    lemming.tick(&terrain_fixture, &mut objects_fixture);

    assert_eq!(
        lemming,
        LemmingState {
            y: CEILING_HIT_Y_RESET,
            frame: 0,
            direction: Direction::Left,
            activity: Activity::Walking,
            animation: LemmingAnimation::Walking,
            ..lemming_fixture
        }
    );
}

#[test]
fn jumper_touches_ceiling() {
    let terrain_fixture = TerrainFixtureBuilder::new(20, 20)
        .with_col(10, MIN_FOOT_Y as u32 + 1, 2, TerrainProps::new())
        .build();

    let mut objects_fixture: Vec<ObjectState> = Vec::new();

    let lemming_fixture =
        LemmingState::fixture(10, MIN_FOOT_Y + 1, Direction::Right, Activity::Jumping);
    let mut lemming = lemming_fixture.clone();

    lemming.tick(&terrain_fixture, &mut objects_fixture);

    assert_eq!(
        lemming,
        LemmingState {
            y: MIN_FOOT_Y,
            frame: 0,
            activity: Activity::Walking,
            animation: LemmingAnimation::Walking,
            ..lemming_fixture
        }
    );
}

#[test]
fn splatting_advances_animation() {
    let terrain_fixture = TerrainFixtureBuilder::new(20, 20).build();

    let mut objects_fixture: Vec<ObjectState> = Vec::new();

    let lemming_fixture = LemmingState::fixture(10, 10, Direction::Right, Activity::Splatting);
    let mut lemming = lemming_fixture.clone();

    let keep = lemming.tick(&terrain_fixture, &mut objects_fixture);

    assert!(keep);
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

    let keep = lemming.tick(&terrain_fixture, &mut objects_fixture);

    assert!(!keep);
}

#[test]
fn frying_advances_animation() {
    let terrain_fixture = TerrainFixtureBuilder::new(20, 20).build();

    let mut objects_fixture: Vec<ObjectState> = Vec::new();

    let lemming_fixture = LemmingState::fixture(10, 10, Direction::Right, Activity::Frying);
    let mut lemming = lemming_fixture.clone();

    let keep = lemming.tick(&terrain_fixture, &mut objects_fixture);

    assert!(keep);
    assert_eq!(
        lemming,
        LemmingState {
            frame: 1,
            ..lemming_fixture
        }
    );
}

#[test]
fn frying_removes_lemming() {
    let terrain_fixture = TerrainFixtureBuilder::new(20, 20).build();

    let mut objects_fixture: Vec<ObjectState> = Vec::new();

    let lemming_fixture = LemmingState {
        frame: LemmingAnimation::Frying.frame_count() - 1,
        ..LemmingState::fixture(10, 10, Direction::Right, Activity::Frying)
    };
    let mut lemming = lemming_fixture.clone();

    let keep = lemming.tick(&terrain_fixture, &mut objects_fixture);

    assert!(!keep);
}

#[test]
fn drowner_moves_in_water_right() {
    let terrain_fixture = TerrainFixtureBuilder::new(20, 20).build();

    let mut objects_fixture: Vec<ObjectState> = Vec::new();

    let lemming_fixture = LemmingState::fixture(10, 10, Direction::Right, Activity::Drowning);
    let mut lemming = lemming_fixture.clone();

    let keep = lemming.tick(&terrain_fixture, &mut objects_fixture);

    assert!(keep);
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
    let terrain_fixture = TerrainFixtureBuilder::new(20, 20).build();

    let mut objects_fixture: Vec<ObjectState> = Vec::new();

    let lemming_fixture = LemmingState::fixture(10, 10, Direction::Left, Activity::Drowning);
    let mut lemming = lemming_fixture.clone();

    let keep = lemming.tick(&terrain_fixture, &mut objects_fixture);

    assert!(keep);
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
    let terrain_fixture = TerrainFixtureBuilder::new(20, 20)
        .with(10 + DROWNER_MIN_WALL_DISTANCE, 10, TerrainProps::new())
        .build();

    let mut objects_fixture: Vec<ObjectState> = Vec::new();

    let lemming_fixture = LemmingState::fixture(10, 10, Direction::Right, Activity::Drowning);
    let mut lemming = lemming_fixture.clone();

    let keep = lemming.tick(&terrain_fixture, &mut objects_fixture);

    assert!(keep);
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
    let terrain_fixture = TerrainFixtureBuilder::new(20, 20)
        .with(10 + DROWNER_MIN_WALL_DISTANCE + 1, 10, TerrainProps::new())
        .build();

    let mut objects_fixture: Vec<ObjectState> = Vec::new();

    let lemming_fixture = LemmingState::fixture(10, 10, Direction::Right, Activity::Drowning);
    let mut lemming = lemming_fixture.clone();

    let keep = lemming.tick(&terrain_fixture, &mut objects_fixture);

    assert!(keep);
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
    let terrain_fixture = TerrainFixtureBuilder::new(20, 20).build();

    let mut objects_fixture: Vec<ObjectState> = Vec::new();

    let lemming_fixture = LemmingState {
        frame: LemmingAnimation::Drowning.frame_count() - 1,
        ..LemmingState::fixture(10, 10, Direction::Right, Activity::Drowning)
    };
    let mut lemming = lemming_fixture.clone();

    let keep = lemming.tick(&terrain_fixture, &mut objects_fixture);

    assert!(!keep);
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
