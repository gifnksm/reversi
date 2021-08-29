use crate::{player::Player, traits::ColorExt, Result};
use reversi_core::{Board, Cell, Color, Game, GameState, Pos};

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

    pub fn state(&self) -> &GameState {
        self.game.state()
    }

    pub fn do_turn(&mut self, color: Color) -> Result<()> {
        let pos = self.player(color).next_move(self.game.board())?;
        self.game.put(pos)?;
        Ok(())
    }

    pub fn print_board(&self, color: Option<Color>) {
        let board = self.game.board();

        eprintln!();
        eprint!(" ");
        for ch in ('A'..).take(board.width() as usize) {
            eprint!(" {}", ch);
        }
        eprintln!();

        for y in 0..board.height() {
            eprint!("{}", y + 1);
            for x in 0..board.width() {
                let pos = Pos::from_xy(x, y).unwrap();
                eprint!(" ");
                match board.get(pos) {
                    Cell::Disk(color) => eprint!("{}", color.mark()),
                    Cell::Wall => eprint!("_"),
                    Cell::Empty => {
                        let ch = match color {
                            Some(color) if board.can_flip(color, pos) => '*',
                            _ => '.',
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
        fn print(board: &Board, target_color: Color, your_color: Option<Color>) {
            let target_mark = target_color.mark();
            eprintln!(
                "  {} : {:2} {}",
                target_mark,
                board.count(Some(target_color)),
                if Some(target_color) == your_color {
                    "(you)"
                } else {
                    " "
                }
            );
        }

        let board = self.game.board();
        print(board, Color::Black, your_color);
        print(board, Color::White, your_color);
        eprintln!();
    }

    pub fn print_result(&self) {
        let board = self.game.board();

        eprintln!();

        let black = board.count(Some(Color::Black));
        let white = board.count(Some(Color::White));
        let winner = match black.cmp(&white) {
            std::cmp::Ordering::Less => Some(self.player(Color::White)),
            std::cmp::Ordering::Equal => None,
            std::cmp::Ordering::Greater => Some(self.player(Color::Black)),
        };
        match winner {
            Some(player) => eprintln!("{} {} wins!", player.color().mark(), player.name()),
            None => eprintln!("DRAW!"),
        }
    }
}
