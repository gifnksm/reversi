use argh::FromArgs;
use rand::{seq::IteratorRandom, Rng};
use reversi_core::{Board, Color, Com, Evaluator, WeightUpdater};
use std::{
    fs::File,
    io::{self, BufReader, BufWriter},
    path::{Path, PathBuf},
};

/// Improve evaluation parameters by reinforcement learning
#[derive(Debug, FromArgs)]
struct Args {
    /// parameter file
    #[argh(option)]
    file: Option<PathBuf>,
    #[argh(positional)]
    num_iteration: u32,
}

type Error = Box<dyn std::error::Error>;

fn main() -> Result<(), Error> {
    let mut rng = rand::thread_rng();

    let args: Args = argh::from_env();
    let evaluator = read_evaluator(&args)?;
    let com = Com::new(4, 12, 12);
    let mut updater = WeightUpdater::new(evaluator);

    let mut history = Vec::with_capacity(64);
    for i in 0..args.num_iteration {
        let board = play_game(&mut rng, updater.evaluator(), &com, &mut history);
        update(&mut updater, &board, &mut history);

        if (i + 1) % 100 == 0 {
            eprintln!("{} / {}", i + 1, args.num_iteration);
            write_evaluator(&args, updater.evaluator())?;
        }
    }
    write_evaluator(&args, updater.evaluator())?;

    Ok(())
}

fn read_evaluator(args: &Args) -> Result<Evaluator, Error> {
    let path = args
        .file
        .clone()
        .unwrap_or_else(|| Path::new("dat").join("evaluator.dat"));
    let file = match File::open(&path) {
        Ok(file) => Some(file),
        Err(e) if e.kind() != io::ErrorKind::NotFound => {
            return Err(e.into());
        }
        _ => None,
    };

    let evaluator = if let Some(file) = file {
        let buf = BufReader::new(file);
        Evaluator::read(buf)?
    } else {
        eprintln!("Evaluator data not found: {}", path.display());
        Evaluator::new()
    };

    Ok(evaluator)
}

fn write_evaluator(args: &Args, evaluator: &Evaluator) -> Result<(), Error> {
    let path = args
        .file
        .clone()
        .unwrap_or_else(|| Path::new("dat").join("evaluator.dat"));
    let file = File::create(&path)?;
    let mut buf = BufWriter::new(file);
    evaluator.write(&mut buf)?;
    Ok(())
}

fn play_game(
    rng: &mut impl Rng,
    evaluator: &Evaluator,
    com: &Com,
    history: &mut Vec<(Board, Color)>,
) -> Board {
    let mut board = Board::new();
    let mut color = Color::Black;

    history.clear();

    for _ in 0..8 {
        if let Some(pos) = board.flip_candidates(color).choose(rng) {
            board = board.flipped(color, pos).1;
            history.push((board, color));
        }
        color = color.reverse();
    }

    loop {
        let pos = if board.count(None) > 12 && rng.gen_ratio(1, 100) {
            board.flip_candidates(color).choose(rng)
        } else {
            let next_move = com.next_move(evaluator, &board, color);
            next_move.best_pos
        };
        match pos {
            Some(pos) => {
                board = board.flipped(color, pos).1;
                history.push((board, color));
                color = color.reverse();
            }
            None => {
                color = color.reverse();
                if !board.can_play(color) {
                    break;
                }
            }
        }
    }
    board
}

fn update(updater: &mut WeightUpdater, board: &Board, history: &mut Vec<(Board, Color)>) {
    let result = updater.evaluator().evaluate(board, Color::Black, true);

    let mut board = *board;
    while board.count(None) < 8 {
        board = history.pop().unwrap().0;
    }

    for _ in (board.count(None) as i32)..(Board::SIZE * Board::SIZE - 12) {
        let (board, color) = history.pop().unwrap();
        if color == Color::Black {
            updater.update(&board, result);
        } else {
            updater.update(&board.reverse(), -result);
        }
    }
}
