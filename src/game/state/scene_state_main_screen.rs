#[derive(Clone, PartialEq)]
pub enum ChoiceMainScreen {
    Play,
    Quit,
}

#[derive(Clone)]
pub struct SceneStateMainScreen {
    pub choice: Option<ChoiceMainScreen>,
}
