#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    UpLeft,
    Up,
    UpRight,
    Left,
    Right,
    DownLeft,
    Down,
    DownRight,
}

impl Direction {
    pub const ALL: [Direction; 8] = [
        Self::UpLeft,
        Self::Up,
        Self::UpRight,
        Self::Left,
        Self::Right,
        Self::DownLeft,
        Self::Down,
        Self::DownRight,
    ];
}
