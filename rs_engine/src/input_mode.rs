#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EInputMode {
    Game,
    UI,
    GameUI,
}

impl EInputMode {
    pub fn is_interact_ui(&self) -> bool {
        match self {
            EInputMode::Game => false,
            EInputMode::UI => true,
            EInputMode::GameUI => true,
        }
    }
}
