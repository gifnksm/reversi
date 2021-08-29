pub use self::{color::*, direction::*, pos::*};

mod color;
mod direction;
mod pos;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Board {
    black: PosSet,
    white: PosSet,
    wall: PosSet,
    width: i32,
    height: i32,
}

impl Default for Board {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum WithSizeError {
    #[error("invalid size: ({0},{1})")]
    InvalidSize(i32, i32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cell {
    Disk(Color),
    Wall,
    Empty,
}

impl Board {
    pub const SIZE: i32 = 8;

    pub fn new() -> Self {
        Self::with_size(Board::SIZE, Board::SIZE).unwrap()
    }

    pub fn with_size(width: i32, height: i32) -> Result<Self, WithSizeError> {
        if width > Board::SIZE || height > Board::SIZE {
            return Err(WithSizeError::InvalidSize(width, height));
        }

        fn init_set(width: i32, height: i32) -> Option<(PosSet, PosSet)> {
            let (x0, y0) = (width / 2 - 1, height / 2 - 1);
            let up_left = Pos::from_xy(x0, y0)?;
            let up_right = Pos::from_xy(x0 + 1, y0)?;
            let down_left = Pos::from_xy(x0, y0 + 1)?;
            let down_right = Pos::from_xy(x0 + 1, y0 + 1)?;
            let black_set = up_right | down_left;
            let white_set = up_left | down_right;
            Some((black_set, white_set))
        }

        let (black, white) =
            init_set(width, height).ok_or(WithSizeError::InvalidSize(width, height))?;

        let mut wall = PosSet::new();
        for y in 0..Board::SIZE {
            for x in 0..Board::SIZE {
                if x >= width || y >= height {
                    wall |= Pos::from_xy(x, y).unwrap();
                }
            }
        }

        Ok(Board {
            black,
            white,
            wall,
            width,
            height,
        })
    }

    pub fn width(&self) -> i32 {
        self.width
    }

    pub fn height(&self) -> i32 {
        self.height
    }

    pub fn get(&self, pos: Pos) -> Cell {
        assert!((self.black & self.white).is_empty());
        if self.black.contains(&pos) {
            Cell::Disk(Color::Black)
        } else if self.white.contains(&pos) {
            Cell::Disk(Color::White)
        } else if self.wall.contains(&pos) {
            Cell::Wall
        } else {
            Cell::Empty
        }
    }

    pub fn set(&mut self, pos: Pos, color: Color) {
        assert!(!self.wall.contains(&pos));
        let (insert_mask, remove_mask) = match color {
            Color::Black => (&mut self.black, &mut self.white),
            Color::White => (&mut self.white, &mut self.black),
        };
        let mask = PosSet::new() | pos;
        *insert_mask |= mask;
        *remove_mask &= !mask;
    }

    pub fn count(&self, color: Option<Color>) -> u32 {
        match color {
            Some(Color::Black) => self.black.count(),
            Some(Color::White) => self.white.count(),
            None => {
                (Board::SIZE * Board::SIZE) as u32
                    - self.white.count()
                    - self.black.count()
                    - self.wall.count()
            }
        }
    }

    pub fn flipped(&self, color: Color, pos: Pos) -> (usize, Self) {
        if (self.black | self.white | self.wall).contains(&pos) {
            return (0, *self);
        }

        let mut res = *self;
        let (self_set, other_set) = match color {
            Color::Black => (&mut res.black, &mut res.white),
            Color::White => (&mut res.white, &mut res.black),
        };
        let mut count = 0;
        let mut flip_set = PosSet::new();
        for dir in Direction::ALL {
            let (c, s) = line_flipped(self_set, other_set, pos, dir);
            count += c;
            flip_set |= s;
        }
        if count > 0 {
            *self_set |= flip_set | pos;
            *other_set &= !flip_set;
            count += 1;
        }
        (count, res)
    }

    pub fn can_flip(&self, color: Color, pos: Pos) -> bool {
        if (self.black | self.white | self.wall).contains(&pos) {
            return false;
        }

        let (self_set, other_set) = match color {
            Color::Black => (self.black, self.white),
            Color::White => (self.white, self.black),
        };
        Direction::ALL.iter().any(|dir| {
            let (c, _m) = line_flipped(&self_set, &other_set, pos, *dir);
            c > 0
        })
    }

    pub fn flip_candidates(&self, color: Color) -> impl Iterator<Item = Pos> + '_ {
        let other_set = match color {
            Color::Black => self.white,
            Color::White => self.black,
        };
        let candidates = !(self.black | self.white | self.wall) & other_set.neighbors();
        candidates
            .into_iter()
            .filter(move |pos| self.can_flip(color, *pos))
    }

    pub fn can_play(&self, color: Color) -> bool {
        self.flip_candidates(color).next().is_some()
    }
}

fn line_flipped(
    self_set: &PosSet,
    other_set: &PosSet,
    origin: Pos,
    dir: Direction,
) -> (usize, PosSet) {
    let mut flipped = PosSet::new();
    for (count, pos) in origin.line(dir).enumerate() {
        if other_set.contains(&pos) {
            flipped |= pos;
            continue;
        }
        if self_set.contains(&pos) {
            return (count, flipped);
        }
        break;
    }
    (0, PosSet::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flip_candidates() {
        use Pos as P;

        let board = Board::new();
        assert!(board
            .flip_candidates(Color::Black)
            .eq([P::C4, P::D3, P::E6, P::F5]));

        assert_eq!(board.flipped(Color::Black, P::A1), (0, board));
        let (count, board) = board.flipped(Color::Black, P::D3);
        assert_eq!(count, 2);
        assert!(board.black.into_iter().eq([P::D3, P::D4, P::D5, P::E4]));
        assert!(board.white.into_iter().eq([P::E5]));

        assert!(board
            .flip_candidates(Color::White)
            .eq([P::C3, P::C5, P::E3]));
        let (count, board) = board.flipped(Color::White, P::C5);
        assert_eq!(count, 2);
        assert!(board.black.into_iter().eq([P::D3, P::D4, P::E4]));
        assert!(board.white.into_iter().eq([P::C5, P::D5, P::E5]));

        assert!(board
            .flip_candidates(Color::Black)
            .eq([P::B6, P::C6, P::D6, P::E6, P::F6]));
    }
}
