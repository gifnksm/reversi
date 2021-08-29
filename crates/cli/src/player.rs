use std::time::Instant;

use crate::Result;
use reversi_core::{Board, Color, Com, NextMove, Pos};

pub trait Player {
    fn name(&self) -> &str;
    fn color(&self) -> Color;
    fn next_move(&self, board: &Board) -> Result<Pos>;
}

#[derive(Debug)]
pub struct Human {
    color: Color,
}

impl Human {
    pub fn new(color: Color) -> Self {
        Self { color }
    }
}

impl Player for Human {
    fn name(&self) -> &str {
        "Human"
    }

    fn color(&self) -> Color {
        self.color
    }

    fn next_move(&self, board: &Board) -> Result<Pos> {
        let candidate = board.flip_candidates(self.color).next().unwrap();

        crate::read_input("Input position to put a disk", Some(candidate), &[], |s| {
            let pos = s.parse()?;
            if !board.can_flip(self.color, pos) {
                return Err(format!("cannot put a disk at{}", pos).into());
            }
            Ok(pos)
        })
    }
}

#[derive(Debug)]
pub struct Computer {
    color: Color,
    com: Com,
}

impl Computer {
    pub fn new(color: Color) -> Self {
        Self {
            color,
            com: Com::new(12, 12, 12),
        }
    }
}

impl Player for Computer {
    fn name(&self) -> &str {
        "Computer"
    }

    fn color(&self) -> Color {
        self.color
    }

    fn next_move(&self, board: &Board) -> Result<Pos> {
        eprintln!("Computer thinking...");
        let start = Instant::now();
        let NextMove {
            best_pos,
            score,
            visited_nodes,
        } = self.com.next_move(board, self.color);
        let elapsed = start.elapsed();
        let best_pos = best_pos.ok_or("cannot find a pos to put")?;

        eprintln!("Computer's choice: {}", best_pos);
        eprintln!("Evaluation score: {}", score);
        eprintln!("  Thinking time: {:.2}", elapsed.as_secs_f64());
        eprintln!("  # of nodes: {}", visited_nodes);
        eprintln!(
            "  kNPS: {:.2}",
            visited_nodes as f64 / elapsed.as_secs_f64() / 1000.0
        );

        Ok(best_pos)
    }
}
