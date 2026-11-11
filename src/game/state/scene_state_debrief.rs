#[derive(Clone)]
pub enum ChoiceDebrief {
    Continue,
}

#[derive(Clone)]
pub struct SceneStateDebrief {
    pub choice: Option<ChoiceDebrief>,
}
