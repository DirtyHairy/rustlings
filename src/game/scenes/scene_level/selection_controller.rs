use crate::state::Selection;

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
            SelectionMode::Primary => self.primary_lemming,
            SelectionMode::Secondary => self.secondary_lemming,
        }
    }
}
