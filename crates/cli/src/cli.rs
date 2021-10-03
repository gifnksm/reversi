use crate::{player::Player, traits::ColorExt, Result};
use reversi_core::{Board, Color, Game, Pos};

pub struct Cli {
    game: Game,
    black_player: Box<dyn Player>,
    white_player: Box<dyn Player>,
}

impl Cli {
    pub fn new(game: Game, black_player: Box<dyn Player>, white_player: Box<dyn Player>) -> Self {
        Self {
            game,
            black_player,
            white_player,
        }
    }

    pub fn player(&self, color: Color) -> &dyn Player {
        match color {
            Color::Black => &*self.black_player,
            Color::White => &*self.white_player,
        }
    }

    pub fn player_mut(&mut self, color: Color) -> &mut dyn Player {
        match color {
            Color::Black => &mut *self.black_player,
            Color::White => &mut *self.white_player,
        }
    }

    pub fn turn(&self) -> u32 {
        self.game.turn()
    }

    pub fn turn_color(&self) -> Option<Color> {
        self.game.turn_color()
    }

    pub fn do_turn(&mut self, color: Color) -> Result<()> {
        let board = *self.game.board();
        let pos = self.player_mut(color).next_move(&board)?;
        self.game.put_disk(pos)?;
        Ok(())
    }

    pub fn print_board(&self) {
        eprintln!();
        eprint!(" ");
        for ch in ('A'..).take(Board::SIZE as usize) {
            eprint!(" {}", ch);
        }
        eprintln!();

        for y in 0..Board::SIZE {
            eprint!("{}", y + 1);
            for x in 0..Board::SIZE {
                let pos = Pos::from_xy(x, y).unwrap();
                eprint!(" ");
                match self.game.get_disk(pos) {
                    Some(color) => eprint!("{}", color.mark()),
                    None => {
                        let ch = if self.game.board().can_flip(pos) {
                            '*'
                        } else {
                            '.'
                        };
                        eprint!("{}", ch);
                    }
                }
            }
            eprintln!();
        }
        eprintln!();
    }

    pub fn print_score(&self, your_color: Option<Color>) {
        for target_color in [Color::Black, Color::White] {
            let target_mark = target_color.mark();
            eprintln!(
                "  {} : {:2} {}",
                target_mark,
                self.game.count_disk(Some(target_color)),
                if Some(target_color) == your_color {
                    "(you)"
                } else {
                    " "
                }
            );
        }
        eprintln!();
    }

    pub fn print_result(&self) {
        eprintln!();

        let black = self.game.count_disk(Some(Color::Black));
        let white = self.game.count_disk(Some(Color::White));
        let winner = match black.cmp(&white) {
            std::cmp::Ordering::Less => Some(self.player(Color::White)),
            std::cmp::Ordering::Equal => None,
            std::cmp::Ordering::Greater => Some(self.player(Color::Black)),
        };
        match winner {
            Some(player) => eprintln!("{} {} wins!", player.color().mark(), player.name()),
            None => eprintln!("DRAW!"),
        }

        eprintln!();

        self.black_player.print_summary();
        self.white_player.print_summary();
    }
}
