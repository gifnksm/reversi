use super::{Evaluate, DISK_VALUE};
use reversi_core::{Board, Color};

#[derive(Debug, Default, Clone)]
pub struct CountEvaluator {}

impl CountEvaluator {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Evaluate for CountEvaluator {
    fn evaluate(&self, board: &Board, color: Color, _game_over: bool) -> i32 {
        i32::from(DISK_VALUE)
            * ((board.count(Some(color)) as i32) - (board.count(Some(color.reverse())) as i32))
    }
}
