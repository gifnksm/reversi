use reversi_core::Board;

pub use self::{count::*, weight::*};

mod count;
mod weight;

const DISK_VALUE: i16 = 1000;

pub trait Evaluate {
    fn evaluate(&self, board: &Board, game_over: bool) -> i32;
}
