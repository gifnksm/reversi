use crate::{Board, Color, Pos};

#[derive(Debug, Clone)]
pub struct Game {
    state: GameState,
    board: Board,
    history: Vec<Board>,
}

#[derive(Debug, Clone, Copy)]
pub enum GameState {
    Turn(u32, Color),
    GameOver(u32),
}

impl Default for Game {
    fn default() -> Self {
        Self {
            state: GameState::Turn(1, Color::Black),
            board: Board::default(),
            history: vec![],
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PutError {
    #[error("already game over")]
    GameOver,
    #[error("cannot put a disk at {0}")]
    CannotPut(Pos),
}

impl Game {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_board(board: Board) -> Self {
        Self {
            board,
            ..Default::default()
        }
    }

    pub fn state(&self) -> &GameState {
        &self.state
    }

    pub fn board(&self) -> &Board {
        &self.board
    }

    pub fn put(&mut self, pos: Pos) -> Result<(), PutError> {
        let (turn, color) = match self.state {
            GameState::Turn(turn, color) => (turn, color),
            GameState::GameOver(_) => return Err(PutError::GameOver),
        };

        let (count, flipped) = self.board.flipped(color, pos);
        if count == 0 {
            return Err(PutError::CannotPut(pos));
        }

        self.history.push(self.board);
        self.board = flipped;

        if self.board.can_play(color.reverse()) {
            self.state = GameState::Turn(turn + 1, color.reverse());
            return Ok(());
        }
        if self.board.can_play(color) {
            return Ok(());
        }
        self.state = GameState::GameOver(turn + 1);
        Ok(())
    }
}
