use crate::{
    cli::Cli,
    player::{Computer, ComputerLevel, Human, Player},
    traits::ColorExt,
};
use reversi_core::{Board, Color, Evaluator, Game, GameState};
use std::{
    fmt,
    fs::File,
    io::{self, BufReader},
    path::Path,
};

mod cli;
mod player;
mod traits;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> Result<()> {
    let board = Board::new();
    let game = Game::with_board(board);
    let black_player = choose_player(Color::Black)?;
    let white_player = choose_player(Color::White)?;

    let mut cli = Cli::new(game, black_player, white_player);

    loop {
        match *cli.state() {
            GameState::Turn(turn, color) => {
                let player = cli.player(color);
                eprintln!();
                eprintln!(
                    "=== Turn #{}: {} {}'s Turn ===",
                    turn,
                    color.mark(),
                    player.name()
                );
                cli.print_board(Some(color));
                cli.print_score(Some(color));
                cli.do_turn(color)?;
            }
            GameState::GameOver(turn) => {
                eprintln!();
                eprintln!("=== Turn #{}: Game Over ===", turn);
                cli.print_board(None);
                cli.print_score(None);
                cli.print_result();
                break;
            }
        }
    }

    Ok(())
}

fn read_input<T>(
    prompt: &str,
    default_value: Option<T>,
    candidates: &[(T, &str)],
    mut parser: impl FnMut(&str) -> Result<T>,
) -> Result<T>
where
    T: fmt::Display,
{
    let mut buf = String::new();
    loop {
        if let Some(default_value) = &default_value {
            eprintln!("{} [{}]", prompt, default_value);
        } else {
            eprintln!("{}", prompt);
        }

        if !candidates.is_empty() {
            for (candidate, desc) in candidates {
                eprintln!("  {}: {}", candidate, desc);
            }
        }

        eprint!("> ");
        buf.clear();
        io::stdin().read_line(&mut buf)?;

        let s = buf.trim();
        if s.is_empty() {
            if let Some(default_value) = default_value {
                return Ok(default_value);
            }
        }

        match parser(s) {
            Ok(val) => return Ok(val),
            Err(e) => {
                eprintln!("ERROR: {}", e);
                eprintln!();
                continue;
            }
        };
    }
}

fn choose_player(color: Color) -> Result<Box<dyn Player>> {
    #[derive(Debug, Clone, Copy)]
    enum PlayerKind {
        Human,
        Computer,
    }
    impl fmt::Display for PlayerKind {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                PlayerKind::Human => write!(f, "H"),
                PlayerKind::Computer => write!(f, "C"),
            }
        }
    }

    let candidates = &[
        (PlayerKind::Human, "Human"),
        (PlayerKind::Computer, "Computer"),
    ];

    let kind = read_input(
        &format!("Choose {} player kind", color.mark()),
        Some(PlayerKind::Human),
        candidates,
        |s| {
            let s = s.to_ascii_uppercase();
            match s.as_str() {
                "H" => Ok(PlayerKind::Human),
                "C" => Ok(PlayerKind::Computer),
                _ => Err(format!("Invalid player kind: {}", s).into()),
            }
        },
    )?;

    match kind {
        PlayerKind::Human => Ok(Box::new(Human::new(color))),
        PlayerKind::Computer => {
            let data_path = Path::new("dat").join("evaluator.dat");
            let evaluator = if data_path.exists() {
                let file = File::open(data_path)?;
                let buf = BufReader::new(file);
                Evaluator::read(buf)?
            } else {
                eprintln!("Evaluator data not found: {}", data_path.display());
                Evaluator::new()
            };

            let candidates = &[
                (ComputerLevel::Level1, "Level 1"),
                (ComputerLevel::Level2, "Level 2"),
                (ComputerLevel::Level3, "Level 3"),
                (ComputerLevel::Level4, "Level 4"),
            ];
            let level = read_input(
                &format!("Choose {} player level", color.mark()),
                Some(ComputerLevel::Level4),
                candidates,
                |s| match s {
                    "1" => Ok(ComputerLevel::Level1),
                    "2" => Ok(ComputerLevel::Level2),
                    "3" => Ok(ComputerLevel::Level3),
                    "4" => Ok(ComputerLevel::Level4),
                    _ => Err(format!("Invalid player level: {}", s).into()),
                },
            )?;
            Ok(Box::new(Computer::new(color, evaluator, level)))
        }
    }
}
