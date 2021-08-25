use reversi_core::{Board, Color, Game, GameState, Pos};
use std::io;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> Result<()> {
    let mut game = Game::new();

    while let GameState::Turn(color) = *game.state() {
        print_board(game.board(), Some(color));
        print_score(game.board(), Some(color));

        let pos = read_pos(game.board(), color)?;
        if let Err(e) = game.put(pos) {
            eprintln!("ERROR: {}", e);
        }
    }

    print_board(game.board(), None);
    print_score(game.board(), None);
    print_result(game.board());

    Ok(())
}

fn print_score(board: &Board, color: Option<Color>) {
    fn print(board: &Board, target_color: Color, your_color: Option<Color>) {
        let target_mark = match target_color {
            Color::Black => 'O',
            Color::White => 'X',
        };
        eprintln!(
            "  {} : {:2} {}",
            target_mark,
            board.count(target_color),
            if Some(target_color) == your_color {
                "(you)"
            } else {
                " "
            }
        );
    }
    print(board, Color::Black, color);
    print(board, Color::White, color);
    eprintln!();
}

fn print_board(board: &Board, color: Option<Color>) {
    eprintln!();
    eprint!(" ");
    for ch in 'A'..='H' {
        eprint!(" {}", ch);
    }
    eprintln!();

    for y in 0..Board::SIZE {
        eprint!("{}", y + 1);
        for x in 0..Board::SIZE {
            let pos = Pos::from_xy(x, y).unwrap();
            eprint!(" ");
            match board.get(pos) {
                Some(reversi_core::Color::Black) => eprint!("O"),
                Some(reversi_core::Color::White) => eprint!("X"),
                None => {
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

fn read_pos(board: &Board, color: Color) -> Result<Pos> {
    let candidate = board.flip_candidates(color).next().unwrap();

    eprintln!("Input position to put a disk [{}]", candidate);
    loop {
        eprint!("> ");
        let mut buf = String::new();
        io::stdin().read_line(&mut buf)?;
        let buf = buf.trim();
        if buf.is_empty() {
            return Ok(candidate);
        }
        match buf.parse() {
            Ok(pos) => return Ok(pos),
            Err(e) => eprintln!("ERROR: {}", e),
        }
    }
}

fn print_result(board: &Board) {
    eprintln!();

    let black = board.count(Color::Black);
    let white = board.count(Color::White);
    match black.cmp(&white) {
        std::cmp::Ordering::Less => eprintln!("X wins!"),
        std::cmp::Ordering::Equal => eprintln!("DRAW!"),
        std::cmp::Ordering::Greater => eprintln!("O wins!"),
    }
}
