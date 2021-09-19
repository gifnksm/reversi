use argh::FromArgs;
use reversi_com::WeightEvaluator;
use reversi_core::{Board, Color, Pos};
use std::{
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
};

/// Dump evaluation parameters
#[derive(Debug, FromArgs)]
struct Args {
    /// parameter file
    #[argh(positional, default = "Path::new(\"dat\").join(\"evaluator.dat\")")]
    file: PathBuf,
}

type Error = Box<dyn std::error::Error>;

fn main() -> Result<(), Error> {
    let args: Args = argh::from_env();
    let evaluator = WeightEvaluator::read(BufReader::new(File::open(&args.file)?))?;

    for (name, patterns, weight) in evaluator.weight().patterns() {
        let pattern = &patterns[0];
        let mut sorted = (0..).zip(weight.iter().copied()).collect::<Vec<_>>();
        sorted.sort_by(|(_, a), (_, b)| a.cmp(b).reverse());

        println!("===== {} =====", name);
        println!("TOP 10 BOARDS");
        print_boards(pattern, &sorted[..10]);
        println!();
        println!("WORST 10 BOARDS");
        print_boards(
            pattern,
            &sorted.iter().copied().rev().take(10).collect::<Vec<_>>(),
        );
        println!();
    }
    println!("===== Parity =====");
    println!("Evan: {}", evaluator.weight().parity()[0]);
    println!("Odd:  {}", evaluator.weight().parity()[1]);

    Ok(())
}

fn print_boards(pattern: &[Pos], scores: &[(u16, i16)]) {
    let (x_min, x_max) = pattern
        .iter()
        .map(|p| p.x())
        .fold((i8::MAX, i8::MIN), |(min, max), v| {
            (i8::min(min, v), i8::max(max, v))
        });
    let (y_min, y_max) = pattern
        .iter()
        .map(|p| p.y())
        .fold((i8::MAX, i8::MIN), |(min, max), v| {
            (i8::min(min, v), i8::max(max, v))
        });

    print!(" ");
    for (_index, value) in scores {
        print!(" | {:^16}", value);
    }
    println!(" |");
    print!(" ");
    for _ in 0..10 {
        let mut chunk = String::new();
        for x in x_min..=x_max {
            chunk.push(' ');
            chunk.push((b'A' + x as u8) as char);
        }
        print!(" | {:^16}", chunk);
    }
    println!(" |");
    for y in y_min..=y_max {
        print!("{}", y);
        for (index, _value) in scores {
            let mut chunk = String::new();
            for x in x_min..=x_max {
                let p = Pos::from_xy(x, y).unwrap();
                let board = Board::from_pattern_index(pattern, *index);
                let mark = match board.get(p) {
                    Some(Color::Black) => 'O',
                    Some(Color::White) => 'X',
                    None if pattern.contains(&p) => '_',
                    None => ' ',
                };
                chunk.push(' ');
                chunk.push(mark);
            }
            print!(" | {:^16}", chunk);
        }
        println!(" |");
    }
}
