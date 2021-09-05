use crate::{Board, Color, Evaluate, Pos};

#[derive(Debug, Clone, Copy)]
pub struct NextMove {
    pub best_pos: Option<Pos>,
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

    pub fn next_move(&self, evaluator: &impl Evaluate, board: &Board, color: Color) -> NextMove {
        let left = board.count(None);
        if left <= self.exact_depth {
            self.end_search(evaluator, board, color, left, (-i32::MAX, i32::MAX))
        } else if left <= self.wld_depth {
            self.end_search(evaluator, board, color, left, (-i32::MAX, 1))
        } else {
            let (board, color) = if (color == Color::White && self.mid_depth % 2 == 0)
                || (color == Color::Black && self.mid_depth % 2 == 1)
            {
                (board.reverse(), color.reverse())
            } else {
                (*board, color)
            };
            self.mid_search(evaluator, &board, color, self.mid_depth)
        }
    }

    fn end_search(
        &self,
        evaluator: &impl Evaluate,
        board: &Board,
        color: Color,
        depth: u32,
        (alpha, beta): (i32, i32),
    ) -> NextMove {
        let mut visited_nodes = 0;
        let (score, best_pos) = self.alpha_beta(
            evaluator,
            board,
            color,
            depth,
            (alpha, beta),
            false,
            &mut visited_nodes,
        );
        NextMove {
            best_pos,
            visited_nodes,
            score,
        }
    }

    fn mid_search(
        &self,
        evaluator: &impl Evaluate,
        board: &Board,
        color: Color,
        depth: u32,
    ) -> NextMove {
        let mut visited_nodes = 0;
        let (score, best_pos) = self.alpha_beta(
            evaluator,
            board,
            color,
            depth,
            (-i32::MAX, i32::MAX),
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
        evaluator: &impl Evaluate,
        board: &Board,
        color: Color,
        depth: u32,
        (mut alpha, beta): (i32, i32),
        in_pass: bool,
        visited_nodes: &mut u32,
    ) -> (i32, Option<Pos>) {
        if depth == 0 {
            *visited_nodes += 1;
            return (evaluator.evaluate(board, color, board.game_over()), None);
        }

        let mut has_candidate = false;
        let mut best_pos = None;
        for (pos, board) in board.all_flipped(color) {
            has_candidate = true;
            let value = -self
                .alpha_beta(
                    evaluator,
                    &board,
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
            return (evaluator.evaluate(board, color, true), None);
        }

        (
            -self
                .alpha_beta(
                    evaluator,
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
