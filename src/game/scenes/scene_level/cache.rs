#[derive(Default)]
pub struct Cache {
    pub selected_primary: Option<Option<usize>>,
    pub selected_secondary: Option<Option<usize>>,
}

impl Cache {
    pub fn clear_selection(&mut self) {
        self.selected_primary = None;
        self.selected_secondary = None;
    }
}
