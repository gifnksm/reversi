use reversi_core::Color;

pub trait ColorExt {
    fn mark(&self) -> char;
}

impl ColorExt for Color {
    fn mark(&self) -> char {
        match self {
            Color::Black => 'O',
            Color::White => 'X',
        }
    }
}
