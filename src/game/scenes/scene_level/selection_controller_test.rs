use std::collections::VecDeque;

use rustlings::game_data::LEVEL_HEIGHT;

use super::{SelectionController, SelectionMode};
use crate::{
    scene::{MouseCoordinates, SceneEvent},
    scenes::scene_level::cache::Cache,
    state::{
        Activity, ActivityStateFalling, ActivityStateFloating, LemmingHealth, LemmingState,
        SceneStateLevel, Selection,
    },
};

#[derive(Default)]
struct FixtureBuilder {
    lemmings: VecDeque<LemmingState>,
    level_x: u32,
}

impl FixtureBuilder {
    fn new() -> Self {
        Default::default()
    }

    fn with_lemming(mut self, lemming: LemmingState) -> Self {
        self.lemmings.push_back(LemmingState {
            id: self.lemmings.len() as u32,
            ..lemming
        });

        self
    }

    fn with_lemming_at(self, x: i32, y: i32, activity: Activity, health: LemmingHealth) -> Self {
        self.with_lemming(LemmingState {
            x,
            y,
            activity,
            health,
            ..Default::default()
        })
    }

    fn without_lemming(mut self, id: u32) -> Self {
        self.lemmings.retain(|lemming| lemming.id != id);

        self
    }

    fn with_level_x(mut self, level_x: u32) -> Self {
        self.level_x = level_x;

        self
    }

    fn build(self) -> SceneStateLevel {
        let lemmings_out = self.lemmings.len() as u32;

        SceneStateLevel {
            lemmings: self.lemmings,
            lemmings_out,
            level_x: self.level_x,
            ..Default::default()
        }
    }

    fn build_controller(self) -> (SelectionController, SceneStateLevel, Cache) {
        (SelectionController::new(), self.build(), Cache::default())
    }
}

fn mouse_move(x: u32, y: u32) -> SceneEvent {
    SceneEvent::MouseMove(MouseCoordinates {
        x,
        y,
        x_frac: 0.0,
        y_frac: 0.0,
    })
}

#[test]
fn update_single_healthy_non_prio_sets_secondary() {
    let (mut controller, mut state, mut cache) = FixtureBuilder::new()
        .with_lemming_at(8, 10, Activity::Walking, LemmingHealth::Healthy)
        .build_controller();

    let modified = controller.dispatch_event(mouse_move(3, 2), &mut state, &mut cache);

    assert!(modified);
    assert_eq!(
        state.selection,
        Selection {
            lemming_count: 1,
            primary_lemming: None,
            secondary_lemming: Some(0),
            secondary_lemming_stale: false,
        }
    );
}

#[test]
fn update_single_healthy_prio_sets_primary() {
    let (mut controller, mut state, mut cache) = FixtureBuilder::new()
        .with_lemming_at(8, 10, Activity::Blocking, LemmingHealth::Healthy)
        .build_controller();

    controller.dispatch_event(mouse_move(3, 2), &mut state, &mut cache);

    assert_eq!(
        state.selection,
        Selection {
            lemming_count: 1,
            primary_lemming: Some(0),
            secondary_lemming: None,
            secondary_lemming_stale: true,
        }
    );
}

#[test]
fn update_non_prio_with_ohno_promotes_to_primary() {
    let (mut controller, mut state, mut cache) = FixtureBuilder::new()
        .with_lemming_at(8, 10, Activity::Walking, LemmingHealth::OhNo)
        .build_controller();

    controller.dispatch_event(mouse_move(3, 2), &mut state, &mut cache);

    assert_eq!(
        state.selection,
        Selection {
            lemming_count: 1,
            primary_lemming: Some(0),
            secondary_lemming: None,
            secondary_lemming_stale: true,
        }
    );
}

#[test]
fn update_prio_with_ohno_stays_primary() {
    let (mut controller, mut state, mut cache) = FixtureBuilder::new()
        .with_lemming_at(8, 10, Activity::Blocking, LemmingHealth::OhNo)
        .build_controller();

    controller.dispatch_event(mouse_move(3, 2), &mut state, &mut cache);

    assert_eq!(
        state.selection,
        Selection {
            lemming_count: 1,
            primary_lemming: Some(0),
            secondary_lemming: None,
            secondary_lemming_stale: true,
        }
    );
}

#[test]
fn update_exploding_lemming_is_not_counted() {
    let (mut controller, mut state, mut cache) = FixtureBuilder::new()
        .with_lemming_at(8, 10, Activity::Walking, LemmingHealth::Exploding)
        .build_controller();

    controller.dispatch_event(mouse_move(3, 2), &mut state, &mut cache);

    assert_eq!(
        state.selection,
        Selection {
            lemming_count: 0,
            primary_lemming: None,
            secondary_lemming: None,
            secondary_lemming_stale: true,
        }
    );
}

#[test]
fn update_classifies_prio_activities_as_primary() {
    let prio_activities = [
        Activity::Blocking,
        Activity::Bashing,
        Activity::Digging,
        Activity::Mining,
        Activity::Building,
    ];

    for activity in prio_activities {
        let (mut controller, mut state, mut cache) = FixtureBuilder::new()
            .with_lemming_at(8, 10, activity.clone(), LemmingHealth::Healthy)
            .build_controller();

        controller.dispatch_event(mouse_move(3, 2), &mut state, &mut cache);

        assert_eq!(
            state.selection,
            Selection {
                lemming_count: 1,
                primary_lemming: Some(0),
                secondary_lemming: None,
                secondary_lemming_stale: true,
            },
            "{:?} should be classified as primary",
            activity
        );
    }
}

#[test]
fn update_classifies_non_prio_activities_as_secondary() {
    let non_prio_activities = [
        Activity::Climbing,
        Activity::Floating(ActivityStateFloating::default()),
        Activity::Falling(ActivityStateFalling::default()),
        Activity::Walking,
        Activity::Jumping,
        Activity::Splatting,
        Activity::Drowning,
        Activity::Frying,
        Activity::Exitting,
    ];

    for activity in non_prio_activities {
        let (mut controller, mut state, mut cache) = FixtureBuilder::new()
            .with_lemming_at(8, 10, activity.clone(), LemmingHealth::Healthy)
            .build_controller();

        controller.dispatch_event(mouse_move(3, 2), &mut state, &mut cache);

        assert_eq!(
            state.selection,
            Selection {
                lemming_count: 1,
                primary_lemming: None,
                secondary_lemming: Some(0),
                secondary_lemming_stale: false,
            },
            "{:?} should be classified as secondary",
            activity
        );
    }
}

#[test]
fn update_cursor_at_or_below_level_height_skips_hit_test() {
    // Lemming at y=167 → hitbox_y=157. mouse_y=159 → cursor_y=157 hits;
    // mouse_y=LEVEL_HEIGHT (160) skips the iteration entirely.
    let (mut controller, mut state, mut cache) = FixtureBuilder::new()
        .with_lemming_at(8, 167, Activity::Walking, LemmingHealth::Healthy)
        .build_controller();

    controller.dispatch_event(mouse_move(3, LEVEL_HEIGHT - 1), &mut state, &mut cache);
    assert_eq!(
        state.selection.lemming_count, 1,
        "mouse_y just below LEVEL_HEIGHT should hit"
    );

    controller.dispatch_event(mouse_move(3, LEVEL_HEIGHT), &mut state, &mut cache);
    assert_eq!(
        state.selection.lemming_count, 0,
        "mouse_y at LEVEL_HEIGHT skips hit test"
    );
}

#[test]
fn update_applies_cursor_offset_and_level_x() {
    // For each level_x, place the lemming at world x = 8 + level_x so that
    // its hitbox spans cursor_x ∈ [level_x, level_x + 12]. With mouse_x=3,
    // cursor_x = 0 + level_x = level_x → left-edge hit. With mouse_x=16,
    // cursor_x = level_x + 13 → miss.
    for level_x in [0u32, 50, 200] {
        let (mut controller, mut state, mut cache) = FixtureBuilder::new()
            .with_level_x(level_x)
            .with_lemming_at(
                8 + level_x as i32,
                10,
                Activity::Walking,
                LemmingHealth::Healthy,
            )
            .build_controller();

        controller.dispatch_event(mouse_move(3, 2), &mut state, &mut cache);
        assert_eq!(
            state.selection.lemming_count, 1,
            "level_x={}: mouse_x=3 should hit (cursor_x={})",
            level_x, level_x
        );

        controller.dispatch_event(mouse_move(16, 2), &mut state, &mut cache);
        assert_eq!(
            state.selection.lemming_count,
            0,
            "level_x={}: mouse_x=16 should miss (cursor_x={})",
            level_x,
            level_x + 13
        );
    }
}

#[test]
fn update_multiple_non_prio_picks_last_iterated_as_secondary() {
    let (mut controller, mut state, mut cache) = FixtureBuilder::new()
        .with_lemming_at(8, 10, Activity::Walking, LemmingHealth::Healthy)
        .with_lemming_at(8, 10, Activity::Walking, LemmingHealth::Healthy)
        .build_controller();

    controller.dispatch_event(mouse_move(3, 2), &mut state, &mut cache);

    assert_eq!(
        state.selection,
        Selection {
            lemming_count: 2,
            primary_lemming: None,
            secondary_lemming: Some(1),
            secondary_lemming_stale: false,
        }
    );
}

#[test]
fn update_multiple_prio_picks_last_iterated_as_primary() {
    let (mut controller, mut state, mut cache) = FixtureBuilder::new()
        .with_lemming_at(8, 10, Activity::Blocking, LemmingHealth::Healthy)
        .with_lemming_at(8, 10, Activity::Blocking, LemmingHealth::Healthy)
        .build_controller();

    controller.dispatch_event(mouse_move(3, 2), &mut state, &mut cache);

    assert_eq!(
        state.selection,
        Selection {
            lemming_count: 2,
            primary_lemming: Some(1),
            secondary_lemming: None,
            secondary_lemming_stale: true,
        }
    );
}

#[test]
fn update_mix_sets_both_to_last_of_each_category() {
    let (mut controller, mut state, mut cache) = FixtureBuilder::new()
        .with_lemming_at(8, 10, Activity::Walking, LemmingHealth::Healthy)
        .with_lemming_at(8, 10, Activity::Blocking, LemmingHealth::Healthy)
        .with_lemming_at(8, 10, Activity::Walking, LemmingHealth::Healthy)
        .with_lemming_at(8, 10, Activity::Blocking, LemmingHealth::Healthy)
        .build_controller();

    controller.dispatch_event(mouse_move(3, 2), &mut state, &mut cache);

    assert_eq!(
        state.selection,
        Selection {
            lemming_count: 4,
            primary_lemming: Some(3),
            secondary_lemming: Some(2),
            secondary_lemming_stale: false,
        }
    );
}

#[test]
fn update_secondary_persists_when_cursor_leaves_to_empty() {
    let (mut controller, mut state, mut cache) = FixtureBuilder::new()
        .with_lemming_at(8, 10, Activity::Walking, LemmingHealth::Healthy)
        .build_controller();

    controller.dispatch_event(mouse_move(3, 2), &mut state, &mut cache);
    assert_eq!(
        state.selection,
        Selection {
            lemming_count: 1,
            primary_lemming: None,
            secondary_lemming: Some(0),
            secondary_lemming_stale: false,
        }
    );

    let modified = controller.dispatch_event(mouse_move(200, 2), &mut state, &mut cache);

    assert!(modified);
    assert_eq!(
        state.selection,
        Selection {
            lemming_count: 0,
            primary_lemming: None,
            secondary_lemming: Some(0),
            secondary_lemming_stale: true,
        },
        "secondary_lemming persists when cursor moves to empty (DOS bug)"
    );
}

#[test]
fn update_secondary_overwritten_by_new_non_prio() {
    let (mut controller, mut state, mut cache) = FixtureBuilder::new()
        .with_lemming_at(8, 10, Activity::Walking, LemmingHealth::Healthy)
        .with_lemming_at(28, 10, Activity::Walking, LemmingHealth::Healthy)
        .build_controller();

    controller.dispatch_event(mouse_move(3, 2), &mut state, &mut cache);
    assert_eq!(state.selection.secondary_lemming, Some(0));

    controller.dispatch_event(mouse_move(23, 2), &mut state, &mut cache);

    assert_eq!(
        state.selection,
        Selection {
            lemming_count: 1,
            primary_lemming: None,
            secondary_lemming: Some(1),
            secondary_lemming_stale: false,
        }
    );
}

#[test]
fn update_moving_from_non_prio_onto_prio_only_retains_stale_secondary() {
    let (mut controller, mut state, mut cache) = FixtureBuilder::new()
        .with_lemming_at(8, 10, Activity::Walking, LemmingHealth::Healthy)
        .with_lemming_at(28, 10, Activity::Blocking, LemmingHealth::Healthy)
        .build_controller();

    controller.dispatch_event(mouse_move(3, 2), &mut state, &mut cache);
    assert_eq!(state.selection.secondary_lemming, Some(0));

    controller.dispatch_event(mouse_move(23, 2), &mut state, &mut cache);

    assert_eq!(
        state.selection,
        Selection {
            lemming_count: 1,
            primary_lemming: Some(1),
            secondary_lemming: Some(0),
            secondary_lemming_stale: true,
        },
        "stale secondary retained after moving onto prio-only spot (DOS bug)"
    );
}

#[test]
fn update_hovering_prio_only_leaves_secondary_none() {
    let (mut controller, mut state, mut cache) = FixtureBuilder::new()
        .with_lemming_at(8, 10, Activity::Blocking, LemmingHealth::Healthy)
        .build_controller();

    controller.dispatch_event(mouse_move(3, 2), &mut state, &mut cache);

    assert_eq!(
        state.selection,
        Selection {
            lemming_count: 1,
            primary_lemming: Some(0),
            secondary_lemming: None,
            secondary_lemming_stale: true,
        }
    );
}

#[test]
fn selected_lemming_primary_mode_returns_primary() {
    let (mut controller, mut state, mut cache) = FixtureBuilder::new()
        .with_lemming_at(8, 10, Activity::Blocking, LemmingHealth::Healthy)
        .build_controller();

    controller.dispatch_event(mouse_move(3, 2), &mut state, &mut cache);

    assert_eq!(
        state.selected_lemming(SelectionMode::Primary, &mut cache),
        Some(0)
    );
}

#[test]
fn selected_lemming_primary_mode_falls_back_to_secondary() {
    let (mut controller, mut state, mut cache) = FixtureBuilder::new()
        .with_lemming_at(8, 10, Activity::Walking, LemmingHealth::Healthy)
        .build_controller();

    controller.dispatch_event(mouse_move(3, 2), &mut state, &mut cache);

    assert_eq!(
        state.selected_lemming(SelectionMode::Primary, &mut cache),
        Some(0)
    );
}

#[test]
fn selected_lemming_secondary_mode_ignores_primary() {
    let (mut controller, mut state, mut cache) = FixtureBuilder::new()
        .with_lemming_at(8, 10, Activity::Blocking, LemmingHealth::Healthy)
        .build_controller();

    controller.dispatch_event(mouse_move(3, 2), &mut state, &mut cache);

    assert_eq!(
        state.selected_lemming(SelectionMode::Secondary, &mut cache),
        None
    );
}

#[test]
fn selected_lemming_returns_stale_secondary_when_count_positive() {
    // Sequence: hover non-prio A, then move onto prio-only B.
    // After the second dispatch the state has primary=Some(1), secondary=Some(0)
    // with stale=true, lemming_count=1. selected_lemming ignores the stale flag,
    // preserving the DOS skill-application behavior.
    let (mut controller, mut state, mut cache) = FixtureBuilder::new()
        .with_lemming_at(8, 10, Activity::Walking, LemmingHealth::Healthy)
        .with_lemming_at(28, 10, Activity::Blocking, LemmingHealth::Healthy)
        .build_controller();

    controller.dispatch_event(mouse_move(3, 2), &mut state, &mut cache);
    controller.dispatch_event(mouse_move(23, 2), &mut state, &mut cache);

    assert_eq!(
        state.selected_lemming(SelectionMode::Primary, &mut cache),
        Some(1)
    );
    assert_eq!(
        state.selected_lemming(SelectionMode::Secondary, &mut cache),
        Some(0)
    );
}

#[test]
fn selected_lemming_returns_none_when_count_zero() {
    let (mut controller, mut state, mut cache) = FixtureBuilder::new().build_controller();

    controller.dispatch_event(mouse_move(3, 2), &mut state, &mut cache);

    assert_eq!(
        state.selected_lemming(SelectionMode::Primary, &mut cache),
        None
    );
    assert_eq!(
        state.selected_lemming(SelectionMode::Secondary, &mut cache),
        None
    );
}

#[test]
fn selected_lemming_primary_returns_deque_index_after_removal() {
    let (mut controller, mut state, mut cache) = FixtureBuilder::new()
        .with_lemming_at(8, 10, Activity::Blocking, LemmingHealth::Healthy)
        .with_lemming_at(28, 10, Activity::Blocking, LemmingHealth::Healthy)
        .with_lemming_at(48, 10, Activity::Blocking, LemmingHealth::Healthy)
        .without_lemming(1)
        .build_controller();

    // Hover lemming with id=2, now at deque index 1 after removal.
    controller.dispatch_event(mouse_move(43, 2), &mut state, &mut cache);

    assert_eq!(state.selection.primary_lemming, Some(2));
    assert_eq!(
        state.selected_lemming(SelectionMode::Primary, &mut cache),
        Some(1)
    );
}

#[test]
fn selected_lemming_secondary_returns_deque_index_after_removal() {
    let (mut controller, mut state, mut cache) = FixtureBuilder::new()
        .with_lemming_at(8, 10, Activity::Walking, LemmingHealth::Healthy)
        .with_lemming_at(28, 10, Activity::Walking, LemmingHealth::Healthy)
        .with_lemming_at(48, 10, Activity::Walking, LemmingHealth::Healthy)
        .without_lemming(0)
        .build_controller();

    // Hover lemming with id=2, now at deque index 1 after removal.
    controller.dispatch_event(mouse_move(43, 2), &mut state, &mut cache);

    assert_eq!(state.selection.secondary_lemming, Some(2));
    assert_eq!(
        state.selected_lemming(SelectionMode::Secondary, &mut cache),
        Some(1)
    );
}

#[test]
fn selected_lemming_returns_none_when_selection_id_no_longer_in_deque() {
    let (mut controller, mut state, mut cache) = FixtureBuilder::new()
        .with_lemming_at(8, 10, Activity::Walking, LemmingHealth::Healthy)
        .build_controller();

    controller.dispatch_event(mouse_move(3, 2), &mut state, &mut cache);
    assert_eq!(state.selection.secondary_lemming, Some(0));

    state.lemmings.clear();

    assert_eq!(
        state.selected_lemming(SelectionMode::Secondary, &mut cache),
        None
    );
}

#[test]
fn selected_lemming_for_ui_secondary_returns_none_when_stale() {
    // Same sequence as `selected_lemming_returns_stale_secondary_when_count_positive`.
    // selected_lemming(Secondary) returns the stale id (DOS behavior), but
    // selected_lemming_for_ui(Secondary) gates on `secondary_lemming_stale` and returns None.
    let (mut controller, mut state, mut cache) = FixtureBuilder::new()
        .with_lemming_at(8, 10, Activity::Walking, LemmingHealth::Healthy)
        .with_lemming_at(28, 10, Activity::Blocking, LemmingHealth::Healthy)
        .build_controller();

    controller.dispatch_event(mouse_move(3, 2), &mut state, &mut cache);
    controller.dispatch_event(mouse_move(23, 2), &mut state, &mut cache);

    assert_eq!(
        state.selected_lemming(SelectionMode::Secondary, &mut cache),
        Some(0)
    );
    assert_eq!(
        state.selected_lemming_for_ui(SelectionMode::Secondary, &mut cache),
        None
    );
}

#[test]
fn selected_lemming_for_ui_secondary_returns_fresh_secondary() {
    let (mut controller, mut state, mut cache) = FixtureBuilder::new()
        .with_lemming_at(8, 10, Activity::Walking, LemmingHealth::Healthy)
        .build_controller();

    controller.dispatch_event(mouse_move(3, 2), &mut state, &mut cache);

    assert_eq!(
        state.selected_lemming_for_ui(SelectionMode::Secondary, &mut cache),
        Some(0)
    );
}
