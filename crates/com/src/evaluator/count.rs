use super::{Evaluate, DISK_VALUE};
use reversi_core::{Board, Disk};

#[derive(Debug, Default, Clone)]
pub struct CountEvaluator {}

impl CountEvaluator {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Evaluate for CountEvaluator {
    fn evaluate(&self, board: &Board, _game_over: bool) -> i32 {
        i32::from(DISK_VALUE)
            * ((board.count_disk(Some(Disk::Mine)) as i32)
                - (board.count_disk(Some(Disk::Others)) as i32))
    }
}
