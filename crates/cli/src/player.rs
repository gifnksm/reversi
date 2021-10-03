use crate::{traits::ColorExt, Result};
use rand::prelude::*;
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
        let candidate = board.flip_candidates().into_iter().next().unwrap();

        crate::read_input("Input position to put a disk", Some(candidate), &[], |s| {
            let pos = s.parse()?;
            if !board.can_flip(pos) {
                return Err(format!("cannot put a disk at{}", pos).into());
            }
            Ok(pos)
        })
    }

    fn print_summary(&self) {}
}

#[derive(Debug, Clone, Copy)]
pub enum AiLevel {
    Level1,
    Level2,
    Level3,
    Level4,
}

impl fmt::Display for AiLevel {
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
    pub fn new(color: Color, evaluator: WeightEvaluator, level: AiLevel) -> Self {
        let com = match level {
            AiLevel::Level1 => Com::new(2, 8, 10),
            AiLevel::Level2 => Com::new(4, 10, 12),
            AiLevel::Level3 => Com::new(6, 12, 14),
            AiLevel::Level4 => Com::new(8, 14, 16),
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
            chosen,
            score,
            visited_nodes,
        } = self.com.next_move(&self.evaluator, board);
        let elapsed = start.elapsed();
        let (best_pos, _) = chosen.ok_or("cannot find a pos to put")?;

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

#[derive(Debug)]
pub struct Random {
    color: Color,
    rng: ThreadRng,
}

impl Random {
    pub fn new(color: Color) -> Self {
        Self {
            color,
            rng: rand::thread_rng(),
        }
    }
}

impl Player for Random {
    fn name(&self) -> &str {
        "Random"
    }

    fn color(&self) -> Color {
        self.color
    }

    fn next_move(&mut self, board: &Board) -> Result<Pos> {
        let pos = board
            .flip_candidates()
            .into_iter()
            .choose(&mut self.rng)
            .ok_or("cannot find a pos to put")?;
        Ok(pos)
    }

    fn print_summary(&self) {}
}
