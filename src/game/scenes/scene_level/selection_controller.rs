use rustlings::game_data::LEVEL_HEIGHT;

use crate::{
    scene::{MouseCoordinates, SceneEvent},
    scenes::scene_level::cache::Cache,
    state::{Activity, LemmingHealth, SceneStateLevel},
};

const HITBOX_EXTEND_X: i32 = 12;
const HITBOX_EXTEND_Y: i32 = 12;

const CURSOR_OFFSET_X: i32 = -3;
const CURSOR_OFFSET_Y: i32 = -2;

#[derive(Clone, Copy, PartialEq, Default)]
pub enum SelectionMode {
    #[default]
    Primary,
    Secondary,
}

impl SceneStateLevel {
    fn resolve_lemming(&self, id: u32) -> Option<usize> {
        self.lemmings
            .binary_search_by_key(&id, |lemming| lemming.id)
            .ok()
    }

    pub fn selected_lemming_primary(&self, cache: &mut Cache) -> Option<usize> {
        *cache.selected_primary.get_or_insert_with(|| {
            if self.selection.lemming_count == 0 {
                None
            } else {
                self.selection
                    .primary_lemming
                    .and_then(|index| self.resolve_lemming(index))
            }
        })
    }

    pub fn selected_lemming_secondary(&self, cache: &mut Cache) -> Option<usize> {
        *cache.selected_secondary.get_or_insert_with(|| {
            if self.selection.lemming_count == 0 {
                None
            } else {
                self.selection
                    .secondary_lemming
                    .and_then(|index| self.resolve_lemming(index))
            }
        })
    }

    pub fn selected_lemming(
        &self,
        selection_mode: SelectionMode,
        cache: &mut Cache,
    ) -> Option<usize> {
        match selection_mode {
            SelectionMode::Primary => self
                .selected_lemming_primary(cache)
                .or_else(|| self.selected_lemming_secondary(cache)),

            SelectionMode::Secondary => self.selected_lemming_secondary(cache),
        }
    }
}

#[derive(Default)]
pub struct SelectionController {
    mouse_x: u32,
    mouse_y: u32,
}

impl SelectionController {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn dispatch_event(
        &mut self,
        event: SceneEvent,
        state: &mut SceneStateLevel,
        cache: &mut Cache,
    ) -> bool {
        match event {
            SceneEvent::MouseMove(MouseCoordinates { x, y, .. })
            | SceneEvent::MouseDown(_, MouseCoordinates { x, y, .. })
            | SceneEvent::MouseUp(_, MouseCoordinates { x, y, .. }) => {
                self.mouse_x = x;
                self.mouse_y = y;

                self.update(state, cache)
            }
            _ => false,
        }
    }

    pub fn update(&self, state: &mut SceneStateLevel, cache: &mut Cache) -> bool {
        let mut selection = state.selection;

        selection.lemming_count = 0;
        selection.primary_lemming = None;
        // DOS selection bug: secondary_lemming is not reset

        if self.mouse_y < LEVEL_HEIGHT {
            let cursor_x = self.mouse_x as i32 + CURSOR_OFFSET_X + state.level_x as i32;
            let cursor_y = self.mouse_y as i32 + CURSOR_OFFSET_Y;

            for lemming in state.lemmings.iter() {
                let (foot_x, foot_y) = lemming.animation.foot();

                let hitbox_x = lemming.x - foot_x as i32;
                let hitbox_y = lemming.y - foot_y as i32;

                if cursor_x < hitbox_x
                    || cursor_x > (hitbox_x + HITBOX_EXTEND_X)
                    || cursor_y < hitbox_y
                    || cursor_y >= (hitbox_y + HITBOX_EXTEND_Y)
                    || lemming.health == LemmingHealth::Exploding
                {
                    continue;
                }

                selection.lemming_count += 1;

                match lemming.activity {
                    Activity::Blocking
                    | Activity::Bashing
                    | Activity::Digging
                    | Activity::Mining
                    | Activity::Building => {
                        selection.primary_lemming = Some(lemming.id);
                    }

                    _ => {
                        if lemming.health == LemmingHealth::OhNo {
                            selection.primary_lemming = Some(lemming.id);
                        } else {
                            selection.secondary_lemming = Some(lemming.id);
                        }
                    }
                }
            }
        }

        let modified = state.selection != selection;
        state.selection = selection;

        if modified {
            cache.clear_selection();
        }

        modified
    }
}
