use argh::FromArgs;
use rand::{seq::IteratorRandom, Rng};
use reversi_core::{Board, Color, Com, Evaluator, WeightUpdater};
use std::{
    fmt,
    fs::File,
    io::{self, BufReader, BufWriter},
    path::{Path, PathBuf},
    time::{Duration, Instant},
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

#[derive(Debug, Clone)]
struct Summary {
    start: Instant,
    interval_thinking_time: Duration,
    interval_visited_nodes: u64,
    total_thinking_time: Duration,
    total_visited_nodes: u64,
}

impl Summary {
    fn new() -> Self {
        Self {
            interval_thinking_time: Duration::ZERO,
            interval_visited_nodes: 0,
            start: Instant::now(),
            total_thinking_time: Duration::ZERO,
            total_visited_nodes: 0,
        }
    }

    fn add(&mut self, elapsed: Duration, visited_nodes: u32) {
        self.total_thinking_time += elapsed;
        self.total_visited_nodes += u64::from(visited_nodes);
        self.interval_thinking_time += elapsed;
        self.interval_visited_nodes += u64::from(visited_nodes);
    }

    fn print_iteration(&mut self, current_iteration: u32, total_iteration: u32) {
        let elapsed = self.start.elapsed();
        let progress = f64::from(current_iteration) / f64::from(total_iteration);

        eprintln!(
            "{:8} / {:8} ({:.3}%) (Estimated: {} / {}) ({} nodes, {:.3} sec, {:.2} kNPs)",
            current_iteration,
            total_iteration,
            progress,
            DurationDisplay(elapsed),
            DurationDisplay(elapsed.div_f64(progress)),
            self.interval_visited_nodes,
            self.interval_thinking_time.as_secs_f64(),
            self.interval_visited_nodes as f64 / self.interval_thinking_time.as_secs_f64() / 1000.0
        );

        self.interval_thinking_time = Duration::ZERO;
        self.interval_visited_nodes = 0;
    }

    fn print_total(self) {
        let elapsed = self.start.elapsed();
        eprintln!(
            "Completed! {} ({} nodes, {}, {:.2} kNPs)",
            DurationDisplay(elapsed),
            self.total_visited_nodes,
            DurationDisplay(self.total_thinking_time),
            self.total_visited_nodes as f64 / self.total_thinking_time.as_secs_f64() / 1000.0
        );
    }
}

#[derive(Debug)]
struct DurationDisplay(Duration);

impl fmt::Display for DurationDisplay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let total_ms = self.0.as_millis();
        let ms = total_ms % 1000;
        let total_sec = total_ms / 1000;
        let sec = total_sec % 60;
        let total_min = total_sec / 60;
        let min = total_min % 60;
        let hour = total_min / 60;
        write!(f, "{:02}:{:02}:{:02}.{:03}", hour, min, sec, ms)
    }
}

type Error = Box<dyn std::error::Error>;

fn main() -> Result<(), Error> {
    let mut rng = rand::thread_rng();

    let args: Args = argh::from_env();
    let evaluator = read_evaluator(&args)?;
    let com = Com::new(4, 12, 12);
    let mut updater = WeightUpdater::new(evaluator);
    let mut summary = Summary::new();

    let mut history = Vec::with_capacity(64);
    for i in 0..args.num_iteration {
        let board = play_game(
            &mut rng,
            updater.evaluator(),
            &com,
            &mut history,
            &mut summary,
        );
        update(&mut updater, &board, &mut history);

        if (i + 1) % 100 == 0 {
            write_evaluator(&args, updater.evaluator())?;
            summary.print_iteration(i + 1, args.num_iteration);
        }
    }
    write_evaluator(&args, updater.evaluator())?;
    summary.print_total();

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
    summary: &mut Summary,
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
            let start = Instant::now();
            let next_move = com.next_move(evaluator, &board, color);
            let elapsed = start.elapsed();
            summary.add(elapsed, next_move.visited_nodes);
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

    for _ in (board.count(None) as i8)..(Board::SIZE * Board::SIZE - 12) {
        let (board, color) = history.pop().unwrap();
        if color == Color::Black {
            updater.update(&board, result);
        } else {
            updater.update(&board.reverse(), -result);
        }
    }
}
