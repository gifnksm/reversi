use crate::{Board, Color, Pos};

#[derive(Debug, Clone, Copy)]
pub struct NextMove {
    pub best_pos: Option<Pos>,
    pub visited_nodes: u32,
    pub score: i64,
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

    pub fn next_move(&self, board: &Board, color: Color) -> NextMove {
        let left = board.count(None);
        if left <= self.exact_depth {
            self.end_search(board, color, left)
        } else if left <= self.wld_depth {
            self.wld_search(board, color, left)
        } else {
            self.mid_search(board, color, self.mid_depth)
        }
    }

    fn end_search(&self, board: &Board, color: Color, depth: u32) -> NextMove {
        let mut visited_nodes = 0;
        let (score, best_pos) = self.alpha_beta(
            board,
            color,
            depth,
            (-i64::MAX, i64::MAX),
            false,
            &mut visited_nodes,
        );
        NextMove {
            best_pos,
            visited_nodes,
            score,
        }
    }

    fn wld_search(&self, board: &Board, color: Color, depth: u32) -> NextMove {
        let mut visited_nodes = 0;
        let (score, best_pos) = self.alpha_beta(
            board,
            color,
            depth,
            (-i64::MAX, i64::MAX),
            false,
            &mut visited_nodes,
        );
        NextMove {
            best_pos,
            visited_nodes,
            score,
        }
    }

    fn mid_search(&self, board: &Board, color: Color, depth: u32) -> NextMove {
        let mut visited_nodes = 0;
        let (score, best_pos) = self.alpha_beta(
            board,
            color,
            depth,
            (-i64::MAX, i64::MAX),
            false,
            &mut visited_nodes,
        );
        NextMove {
            best_pos,
            visited_nodes,
            score,
        }
    }

    fn alpha_beta(
        &self,
        board: &Board,
        color: Color,
        depth: u32,
        (mut alpha, beta): (i64, i64),
        in_pass: bool,
        visited_nodes: &mut u32,
    ) -> (i64, Option<Pos>) {
        fn evaluate(board: &Board, color: Color) -> i64 {
            (board.count(Some(color)) as i64) - (board.count(Some(color.reverse())) as i64)
        }

        if depth == 0 {
            *visited_nodes += 1;
            return (evaluate(board, color), None);
        }

        let mut has_candidate = false;
        let mut best_pos = None;
        for pos in board.flip_candidates(color) {
            has_candidate = true;
            let (_, flipped) = board.flipped(color, pos);
            let value = -self
                .alpha_beta(
                    &flipped,
                    color.reverse(),
                    depth - 1,
                    (-beta, -alpha),
                    false,
                    visited_nodes,
                )
                .0;
            if value > alpha {
                alpha = value;
                best_pos = Some((pos, value));
                if alpha >= beta {
                    return (beta, None);
                }
            }
        }

        if let Some((pos, score)) = best_pos {
            return (score, Some(pos));
        }
        if has_candidate {
            return (alpha, None);
        }

        if in_pass {
            *visited_nodes += 1;
            return (evaluate(board, color), None);
        }

        (
            -self
                .alpha_beta(
                    board,
                    color.reverse(),
                    depth,
                    (-beta, -alpha),
                    true,
                    visited_nodes,
                )
                .0,
            None,
        )
    }
}
