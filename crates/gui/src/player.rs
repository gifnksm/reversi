#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct PlayerConf {
    pub(crate) player_kind: PlayerKind,
    pub(crate) computer_kind: ComputerKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PlayerKind {
    Human,
    Computer,
}

impl Default for PlayerKind {
    fn default() -> Self {
        Self::Human
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ComputerKind {
    Random,
    Ai(AiLevel),
}

impl Default for ComputerKind {
    fn default() -> Self {
        Self::Ai(AiLevel::Level4)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AiLevel {
    Level1,
    Level2,
    Level3,
    Level4,
}
