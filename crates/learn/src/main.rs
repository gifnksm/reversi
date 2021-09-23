use argh::FromArgs;
use rand::{seq::IteratorRandom, Rng};
use rayon::prelude::*;
use reversi_com::{Com, Evaluate as _, WeightEvaluator, WeightUpdater};
use reversi_core::{Board, Color};
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
    interval_dist_sum: i32,
    interval_game_count: i32,
    total_thinking_time: Duration,
    total_visited_nodes: u64,
    current_iteration: u32,
    total_iteration: u32,
}

impl Summary {
    fn new(total_iteration: u32) -> Self {
        Self {
            interval_thinking_time: Duration::ZERO,
            interval_visited_nodes: 0,
            interval_dist_sum: 0,
            interval_game_count: 0,
            start: Instant::now(),
            total_thinking_time: Duration::ZERO,
            total_visited_nodes: 0,
            current_iteration: 0,
            total_iteration,
        }
    }

    fn add_result(&mut self, elapsed: Duration, visited_nodes: u32, dist: i32) {
        self.current_iteration += 1;
        self.total_thinking_time += elapsed;
        self.total_visited_nodes += u64::from(visited_nodes);
        self.interval_thinking_time += elapsed;
        self.interval_visited_nodes += u64::from(visited_nodes);
        self.interval_dist_sum += dist;
        self.interval_game_count += 1;
    }

    fn print_iteration(&mut self) {
        let elapsed = self.start.elapsed();
        let progress = f64::from(self.current_iteration) / f64::from(self.total_iteration);

        eprintln!(
            "{:8} / {:8} ({:5.1}%) (Estimated: {} / {}) ({} nodes, {:.3} sec, {:.2} kNPs) (AVG dist {})",
            self.current_iteration,
            self.total_iteration,
            progress * 100.0,
            DurationDisplay(elapsed),
            DurationDisplay(elapsed.div_f64(progress)),
            self.interval_visited_nodes,
            self.interval_thinking_time.as_secs_f64(),
            self.interval_visited_nodes as f64 / self.interval_thinking_time.as_secs_f64() / 1000.0,
            (self.interval_dist_sum + self.interval_game_count / 2) / self.interval_game_count
        );

        self.interval_thinking_time = Duration::ZERO;
        self.interval_visited_nodes = 0;
        self.interval_dist_sum = 0;
        self.interval_game_count = 0;
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

fn div_ceil(n: u32, m: u32) -> u32 {
    (n + m - 1) / m
}

fn main() -> Result<(), Error> {
    let args: Args = argh::from_env();
    let evaluator = read_evaluator(&args)?;
    let com = Com::new(4, 12, 12);
    let mut updater = WeightUpdater::new(evaluator);

    const FLUSH_INTERVAL: u32 = 10;
    const ITERATION_INTERVAL: u32 = 10 * FLUSH_INTERVAL;

    let total_iteration = div_ceil(args.num_iteration, ITERATION_INTERVAL) * ITERATION_INTERVAL;
    let mut summary = Summary::new(total_iteration);

    for _ in 0..total_iteration / ITERATION_INTERVAL {
        for _ in 0..ITERATION_INTERVAL / FLUSH_INTERVAL {
            let evaluator = updater.evaluator().clone();
            (0..FLUSH_INTERVAL)
                .into_par_iter()
                .map(|_| play_game(&evaluator, &com))
                .collect::<Vec<_>>()
                .into_iter()
                .for_each(|(board, history, elapsed, visited_nodes)| {
                    let avg_dist = update(&mut updater, &board, &history);
                    summary.add_result(elapsed, visited_nodes, avg_dist);
                });
            updater.flush();
        }
        write_evaluator(&args, updater.evaluator())?;
        summary.print_iteration();
    }
    write_evaluator(&args, updater.evaluator())?;
    summary.print_total();

    Ok(())
}

fn read_evaluator(args: &Args) -> Result<WeightEvaluator, Error> {
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
        WeightEvaluator::read(buf)?
    } else {
        eprintln!("WeightEvaluator data not found: {}", path.display());
        WeightEvaluator::new()
    };

    Ok(evaluator)
}

fn write_evaluator(args: &Args, evaluator: &WeightEvaluator) -> Result<(), Error> {
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
    evaluator: &WeightEvaluator,
    com: &Com,
) -> (Board, Vec<(Board, Color)>, Duration, u32) {
    let mut rng = rand::thread_rng();
    let mut board = Board::new();
    let mut color = Color::Black;
    let mut total_duration = Duration::ZERO;
    let mut total_visited_nodes = 0;

    let mut history = Vec::with_capacity(64);

    for _ in 0..8 {
        if let Some(pos) = board.flip_candidates(color).choose(&mut rng) {
            board = board.flipped(color, pos).1;
            history.push((board, color));
        }
        color = color.reverse();
    }

    loop {
        let pos = if board.count(None) > 12 && rng.gen_ratio(1, 100) {
            board.flip_candidates(color).choose(&mut rng)
        } else {
            let start = Instant::now();
            let next_move = com.next_move(evaluator, &board, color);
            let elapsed = start.elapsed();
            total_duration += elapsed;
            total_visited_nodes += next_move.visited_nodes;
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
    (board, history, total_duration, total_visited_nodes)
}

fn update(updater: &mut WeightUpdater, board: &Board, history: &[(Board, Color)]) -> i32 {
    let result = updater.evaluator().evaluate(board, Color::Black, true);

    let mut history = history.iter().rev();
    let mut board = *board;
    while board.count(None) < 8 {
        board = history.next().unwrap().0;
    }

    let mut total_dist = 0;
    let mut count = 0;
    for _ in (board.count(None) as i8)..(Board::SIZE * Board::SIZE - 12) {
        let (board, color) = history.next().unwrap();
        let diff = if *color == Color::Black {
            updater.update(board, result)
        } else {
            updater.update(&board.reverse(), -result)
        };
        total_dist += diff.abs();
        count += 1;
    }
    (total_dist + count / 2) / count
}
