use rustlings::game_data::LEVEL_HEIGHT;

use crate::{
    scene::{MouseCoordinates, SceneEvent},
    state::{Activity, SceneStateLevel, Selection},
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

impl Selection {
    pub fn selected_lemming(&self, selection_mode: SelectionMode) -> Option<usize> {
        if self.lemming_count == 0 {
            return None;
        }

        match selection_mode {
            SelectionMode::Primary => self.primary_lemming.or(self.secondary_lemming),
            SelectionMode::Secondary => self.secondary_lemming,
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

    pub fn dispatch_event(&mut self, event: SceneEvent, state: &mut SceneStateLevel) -> bool {
        match event {
            SceneEvent::MouseMove(MouseCoordinates { x, y, .. })
            | SceneEvent::MouseDown(_, MouseCoordinates { x, y, .. })
            | SceneEvent::MouseUp(_, MouseCoordinates { x, y, .. }) => {
                self.mouse_x = x;
                self.mouse_y = y;

                self.update(state)
            }
            _ => false,
        }
    }

    pub fn update(&self, state: &mut SceneStateLevel) -> bool {
        let mut selection = state.selection;

        selection.lemming_count = 0;
        selection.primary_lemming = None;
        // DOS selection bug: secondary_lemming is not reset

        if self.mouse_y < LEVEL_HEIGHT {
            let cursor_x = self.mouse_x as i32 + CURSOR_OFFSET_X + state.level_x as i32;
            let cursor_y = self.mouse_y as i32 + CURSOR_OFFSET_Y;

            for (i, lemming) in state.lemmings.iter().enumerate() {
                let (foot_x, foot_y) = lemming.animation.foot();

                let hitbox_x = lemming.x - foot_x as i32;
                let hitbox_y = lemming.y - foot_y as i32;

                if cursor_x < hitbox_x
                    || cursor_x > (hitbox_x + HITBOX_EXTEND_X)
                    || cursor_y < hitbox_y
                    || cursor_y >= (hitbox_y + HITBOX_EXTEND_Y)
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
                        if lemming.ohno {
                            selection.secondary_lemming = Some(i);
                        } else {
                            selection.primary_lemming = Some(i);
                        }
                    }

                    _ => selection.secondary_lemming = Some(i),
                }
            }
        }

        let modified = state.selection != selection;
        state.selection = selection;

        modified
    }
}
