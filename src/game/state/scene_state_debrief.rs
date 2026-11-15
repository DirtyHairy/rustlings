#[derive(Clone)]
pub enum ChoiceDebrief {
    Continue,
    MainScreen,
}

#[derive(Clone)]
pub struct SceneStateDebrief {
    pub choice: Option<ChoiceDebrief>,
}
