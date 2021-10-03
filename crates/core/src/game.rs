use crate::{Board, Color, Disk, Pos};

#[derive(Debug, Clone)]
pub struct Game {
    state: GameState,
    board: Board,
    turn_color: Color,
    history: Vec<Board>,
}

#[derive(Debug, Clone, Copy)]
enum GameState {
    Turn,
    GameOver,
}

impl Default for Game {
    fn default() -> Self {
        Self {
            state: GameState::Turn,
            board: Board::default(),
            turn_color: Color::Black,
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

    fn is_game_over(&self) -> bool {
        match self.state {
            GameState::Turn => false,
            GameState::GameOver => true,
        }
    }

    pub fn board(&self) -> &Board {
        &self.board
    }

    pub fn turn(&self) -> u32 {
        self.board.count_all_disks()
    }

    pub fn turn_color(&self) -> Option<Color> {
        if self.is_game_over() {
            return None;
        }
        Some(self.turn_color)
    }

    fn disk2color(&self, disk: Disk) -> Color {
        match disk {
            Disk::Mine => self.turn_color,
            Disk::Others => self.turn_color.reverse(),
        }
    }

    fn color2disk(&self, color: Color) -> Disk {
        if color == self.turn_color {
            Disk::Mine
        } else {
            Disk::Others
        }
    }

    pub fn count_disk(&self, color: Option<Color>) -> u32 {
        let disk = color.map(|color| self.color2disk(color));
        self.board.count_disk(disk)
    }

    pub fn get_disk(&self, pos: Pos) -> Option<Color> {
        self.board.get_disk(pos).map(|disk| self.disk2color(disk))
    }

    pub fn put_disk(&mut self, pos: Pos) -> Result<(), PutError> {
        if self.is_game_over() {
            return Err(PutError::GameOver);
        }

        let (_count, flipped) = self.board.flipped(pos).ok_or(PutError::CannotPut(pos))?;

        self.history.push(self.board);

        self.board = flipped;
        self.turn_color = self.turn_color.reverse();

        for _ in 0..2 {
            if self.board.can_play() {
                return Ok(());
            }
            self.board = self.board.passed();
            self.turn_color = self.turn_color.reverse();
        }

        self.state = GameState::GameOver;
        Ok(())
    }
}
