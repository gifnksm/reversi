use crate::{
    cli::Cli,
    player::{AiLevel, Computer, Human, Player, Random},
    traits::ColorExt,
};
use reversi_com::WeightEvaluator;
use reversi_core::{Board, Color, Game, GameState};
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
            #[derive(Debug, Clone, Copy)]
            enum ComputerKind {
                Ai(AiLevel),
                Random,
            }
            impl fmt::Display for ComputerKind {
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    match self {
                        ComputerKind::Ai(level) => write!(f, "{}", level),
                        ComputerKind::Random => write!(f, "R"),
                    }
                }
            }

            let data_path = Path::new("dat").join("evaluator.dat");
            let evaluator = if data_path.exists() {
                let file = File::open(data_path)?;
                let buf = BufReader::new(file);
                WeightEvaluator::read(buf)?
            } else {
                eprintln!("Evaluator data not found: {}", data_path.display());
                WeightEvaluator::new()
            };

            let candidates = &[
                (ComputerKind::Random, "Random"),
                (ComputerKind::Ai(AiLevel::Level1), "Level 1"),
                (ComputerKind::Ai(AiLevel::Level2), "Level 2"),
                (ComputerKind::Ai(AiLevel::Level3), "Level 3"),
                (ComputerKind::Ai(AiLevel::Level4), "Level 4"),
            ];
            let kind = read_input(
                &format!("Choose {} player computer kind", color.mark()),
                Some(ComputerKind::Ai(AiLevel::Level4)),
                candidates,
                |s| {
                    let s = s.to_ascii_uppercase();
                    match s.as_str() {
                        "1" => Ok(ComputerKind::Ai(AiLevel::Level1)),
                        "2" => Ok(ComputerKind::Ai(AiLevel::Level2)),
                        "3" => Ok(ComputerKind::Ai(AiLevel::Level3)),
                        "4" => Ok(ComputerKind::Ai(AiLevel::Level4)),
                        "R" => Ok(ComputerKind::Random),
                        _ => Err(format!("Invalid player computer kind: {}", s).into()),
                    }
                },
            )?;
            Ok(match kind {
                ComputerKind::Ai(level) => Box::new(Computer::new(color, evaluator, level)),
                ComputerKind::Random => Box::new(Random::new(color)),
            })
        }
    }
}
