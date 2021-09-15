use crate::{traits::ColorExt, Result};
use reversi_com::{Com, NextMove, WeightEvaluator};
use reversi_core::{Board, Color, Pos};
use std::{
    fmt,
    time::{Duration, Instant},
};

pub trait Player {
    fn name(&self) -> &str;
    fn color(&self) -> Color;
    fn next_move(&mut self, board: &Board) -> Result<Pos>;
    fn print_summary(&self);
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

    fn next_move(&mut self, board: &Board) -> Result<Pos> {
        let candidate = board.flip_candidates(self.color).next().unwrap();

        crate::read_input("Input position to put a disk", Some(candidate), &[], |s| {
            let pos = s.parse()?;
            if !board.can_flip(self.color, pos) {
                return Err(format!("cannot put a disk at{}", pos).into());
            }
            Ok(pos)
        })
    }

    fn print_summary(&self) {}
}

#[derive(Debug, Clone, Copy)]
pub enum ComputerLevel {
    Level1,
    Level2,
    Level3,
    Level4,
}

impl fmt::Display for ComputerLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Level1 => write!(f, "1"),
            Self::Level2 => write!(f, "2"),
            Self::Level3 => write!(f, "3"),
            Self::Level4 => write!(f, "4"),
        }
    }
}

#[derive(Debug)]
pub struct Computer {
    color: Color,
    evaluator: WeightEvaluator,
    com: Com,
    total_thinking_time: Duration,
    total_visited_nodes: u64,
}

impl Computer {
    pub fn new(color: Color, evaluator: WeightEvaluator, level: ComputerLevel) -> Self {
        let com = match level {
            ComputerLevel::Level1 => Com::new(2, 8, 10),
            ComputerLevel::Level2 => Com::new(4, 10, 12),
            ComputerLevel::Level3 => Com::new(6, 12, 14),
            ComputerLevel::Level4 => Com::new(8, 14, 16),
        };
        Self {
            color,
            evaluator,
            com,
            total_thinking_time: Duration::ZERO,
            total_visited_nodes: 0,
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

    fn next_move(&mut self, board: &Board) -> Result<Pos> {
        eprintln!("Computer thinking...");
        let start = Instant::now();
        let NextMove {
            best_pos,
            score,
            visited_nodes,
        } = self.com.next_move(&self.evaluator, board, self.color);
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

        self.total_thinking_time += elapsed;
        self.total_visited_nodes += u64::from(visited_nodes);

        Ok(best_pos)
    }

    fn print_summary(&self) {
        eprintln!("{} Computer performance summary:", self.color.mark());
        eprintln!(
            "  Thinking time: {:.2}",
            self.total_thinking_time.as_secs_f64()
        );
        eprintln!("  # of nodes: {}", self.total_visited_nodes);
        eprintln!(
            "  kNPS: {:.2}",
            self.total_visited_nodes as f64 / self.total_thinking_time.as_secs_f64() / 1000.0
        );
        eprintln!();
    }
}
