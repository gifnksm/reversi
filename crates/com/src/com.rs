use crate::Evaluate;
use reversi_core::{Board, Pos};

#[derive(Debug, Clone, Copy)]
pub struct NextMove {
    pub chosen: Option<(Pos, Board)>,
    pub visited_nodes: u32,
    pub score: i32,
}

#[derive(Debug)]
pub struct Com {
    mid_depth: u32,
    wld_depth: u32,
    exact_depth: u32,
}

impl Com {
    pub fn new(mid_depth: u32, wld_depth: u32, exact_depth: u32) -> Self {
        Self {
            mid_depth,
            wld_depth,
            exact_depth,
        }
    }

    pub fn next_move(&self, evaluator: &impl Evaluate, board: &Board) -> NextMove {
        let left = board.count_disk(None);
        if left <= self.exact_depth {
            self.end_search(evaluator, board, left, (-i32::MAX, i32::MAX))
        } else if left <= self.wld_depth {
            self.end_search(evaluator, board, left, (-i32::MAX, 1))
        } else {
            self.mid_search(evaluator, board, self.mid_depth)
        }
    }

    fn end_search(
        &self,
        evaluator: &impl Evaluate,
        board: &Board,
        depth: u32,
        (alpha, beta): (i32, i32),
    ) -> NextMove {
        let alpha_beta = alpha_beta::<_, true>;
        let mut visited_nodes = 0;
        let (score, chosen) = alpha_beta(
            evaluator,
            board,
            depth,
            (alpha, beta),
            false,
            &mut visited_nodes,
        );
        NextMove {
            chosen,
            visited_nodes,
            score,
        }
    }

    fn mid_search(&self, evaluator: &impl Evaluate, board: &Board, depth: u32) -> NextMove {
        let alpha_beta = alpha_beta::<_, false>;

        let mut visited_nodes = 0;
        let (score, chosen) = alpha_beta(
            evaluator,
            board,
            depth,
            (-i32::MAX, i32::MAX),
            false,
            &mut visited_nodes,
        );
        NextMove {
            chosen,
            visited_nodes,
            score,
        }
    }
}

fn alpha_beta<E, const END_SEARCH: bool>(
    evaluator: &E,
    board: &Board,
    depth: u32,
    (mut alpha, beta): (i32, i32),
    in_pass: bool,
    visited_nodes: &mut u32,
) -> (i32, Option<(Pos, Board)>)
where
    E: Evaluate,
{
    let alpha_beta = alpha_beta::<E, END_SEARCH>;

    if depth == 0 {
        *visited_nodes += 1;
        let game_over = END_SEARCH;
        return (evaluator.evaluate(board, game_over), None);
    }

    let mut has_candidate = false;
    let mut chosen = None;
    for (pos, flipped) in board.all_flipped() {
        has_candidate = true;
        let value = -alpha_beta(
            evaluator,
            &flipped,
            depth - 1,
            (-beta, -alpha),
            false,
            visited_nodes,
        )
        .0;
        if value > alpha {
            alpha = value;
            chosen = Some((pos, flipped, value));
            if alpha >= beta {
                return (beta, None);
            }
        }
    }

    if let Some((pos, flipped, score)) = chosen {
        return (score, Some((pos, flipped)));
    }
    if has_candidate {
        return (alpha, None);
    }

    if in_pass {
        *visited_nodes += 1;
        return (evaluator.evaluate(board, true), None);
    }

    (
        -alpha_beta(
            evaluator,
            &board.reverse(),
            depth,
            (-beta, -alpha),
            true,
            visited_nodes,
        )
        .0,
        None,
    )
}

#[cfg(test)]
fn nega_max<E, const END_SEARCH: bool>(
    evaluator: &E,
    board: &Board,
    depth: u32,
    in_pass: bool,
    visited_nodes: &mut u32,
) -> (i32, Option<(Pos, Board)>)
where
    E: Evaluate,
{
    let nega_max = nega_max::<E, END_SEARCH>;

    if depth == 0 {
        *visited_nodes += 1;
        let game_over = END_SEARCH;
        return (evaluator.evaluate(board, game_over), None);
    }

    let mut max = i32::MIN;
    let mut has_candidate = false;
    let mut best_pos = None;
    for (pos, flipped) in board.all_flipped() {
        has_candidate = true;
        let value = -nega_max(evaluator, &flipped, depth - 1, false, visited_nodes).0;
        if value > max {
            max = value;
            best_pos = Some((pos, flipped, value));
        }
    }

    if let Some((pos, flipped, score)) = best_pos {
        return (score, Some((pos, flipped)));
    }
    if has_candidate {
        return (max, None);
    }

    if in_pass {
        *visited_nodes += 1;
        return (evaluator.evaluate(board, true), None);
    }

    (
        -nega_max(evaluator, &board.reverse(), depth, true, visited_nodes).0,
        None,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CountEvaluator;
    use reversi_core::Disk;

    #[derive(Debug)]
    struct DummyEvaluator(CountEvaluator);

    impl Evaluate for DummyEvaluator {
        fn evaluate(&self, board: &Board, _game_over: bool) -> i32 {
            let mut value = 0;
            for (disk, v) in board.disks().zip(0..) {
                match disk {
                    Some(Disk::Mine) => value += v,
                    Some(Disk::Others) => value -= v,
                    None => {}
                }
            }
            value
        }
    }

    #[test]
    fn comp_com() {
        let alpha_beta = alpha_beta::<_, false>;
        let nega_max = nega_max::<_, false>;
        let evaluator = DummyEvaluator(CountEvaluator::new());
        let depth = 3;

        let ab = |board| {
            let mut visited_nodes = 0;
            let pos = alpha_beta(
                &evaluator,
                &board,
                depth,
                (-i32::MAX, i32::MAX),
                false,
                &mut visited_nodes,
            );
            (visited_nodes, pos)
        };
        let nb = |board| {
            let mut visited_nodes = 0;
            let pos = nega_max(&evaluator, &board, depth, false, &mut visited_nodes);
            (visited_nodes, pos)
        };

        let mut board = Board::new();
        let mut in_pass = false;
        loop {
            let (alpha_nodes, alpha_pos) = ab(board);
            let (nega_nodes, nega_pos) = nb(board);
            assert!(alpha_nodes <= nega_nodes);
            assert_eq!(alpha_pos, nega_pos);
            match alpha_pos.1 {
                Some((_pos, flipped)) => {
                    board = flipped;
                    in_pass = false;
                }
                None if in_pass => break,
                None => {
                    in_pass = true;
                    board = board.reverse();
                }
            }
        }
    }
}
