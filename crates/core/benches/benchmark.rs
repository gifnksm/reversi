use criterion::{black_box, criterion_group, criterion_main, Criterion};
use reversi_core::{Board, Color, Pos};

fn board_flipped(c: &mut Criterion) {
    c.bench_function("flipped initial", |b| {
        b.iter(|| {
            let board = black_box(Board::new());
            black_box(board.flipped(Color::Black, Pos::F5));
        });
    });

    c.bench_function("flipped many", |b| {
        // o o o o o o o o      o o o o o o o o
        // o x x x x x x o      o O x O x O x o
        // o x x x x x x o      o x O O O x x o
        // o x x _ x x x o      o O O O O O O o
        // o x x x x x x o  =>  o x O O O x x o
        // o x x x x x x o      o O x O x O x o
        // o x x x x x x o      o x x O x x O o
        // o o o o o o o o      o o o o o o o o
        let mut board = Board::empty();
        for x in 0..Board::SIZE {
            for y in 0..Board::SIZE {
                let p = Pos::from_xy(x, y).unwrap();
                board.set(p, Color::Black);
            }
        }
        for x in 1..(Board::SIZE - 1) {
            for y in 1..(Board::SIZE - 1) {
                let p = Pos::from_xy(x, y).unwrap();
                board.set(p, Color::White);
            }
        }
        board.unset(Pos::D4);
        assert_eq!(board.count(Some(Color::Black)), 28);
        assert_eq!(board.count(Some(Color::White)), 35);
        assert_eq!(board.count(None), 1);

        let flipped = board.flipped(Color::Black, Pos::D4);
        assert_eq!(flipped.0, 20);
        assert_eq!(flipped.1.count(Some(Color::Black)), 48);
        assert_eq!(flipped.1.count(Some(Color::White)), 16);

        b.iter(|| {
            let board = black_box(board);
            black_box(board.flipped(Color::Black, Pos::D4));
        });
    });
}

fn board_all_flipped(c: &mut Criterion) {
    c.bench_function("all_flipped initial", |b| {
        b.iter(|| {
            let board = black_box(Board::new());
            for (pos, board) in board.all_flipped(Color::Black) {
                black_box((pos, board));
            }
        });
    });

    c.bench_function("all_flipped many", |b| {
        let mut board = Board::empty();
        for x in 1..(Board::SIZE - 1) {
            for y in 1..(Board::SIZE - 1) {
                let p = Pos::from_xy(x, y).unwrap();
                board.set(p, Color::White);
            }
        }
        for x in 2..(Board::SIZE - 2) {
            for y in 2..(Board::SIZE - 2) {
                let p = Pos::from_xy(x, y).unwrap();
                board.set(p, Color::Black);
            }
        }
        b.iter(|| {
            let board = black_box(board);
            for (pos, board) in board.all_flipped(Color::Black) {
                black_box((pos, board));
            }
        });
    });
}

criterion_group!(benches, board_flipped, board_all_flipped);
criterion_main!(benches);
