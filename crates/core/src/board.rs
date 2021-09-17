pub use self::{color::*, pos::*};

mod color;
mod pos;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Board {
    black: PosSet,
    white: PosSet,
}

impl Default for Board {
    fn default() -> Self {
        Self::new()
    }
}

impl Board {
    pub const SIZE: i8 = 8;

    pub fn new() -> Self {
        Self {
            black: PosSet::new() | Pos::E4 | Pos::D5,
            white: PosSet::new() | Pos::D4 | Pos::E5,
        }
    }

    pub fn empty() -> Self {
        Self {
            black: PosSet::new(),
            white: PosSet::new(),
        }
    }

    pub fn get(&self, pos: Pos) -> Option<Color> {
        debug_assert!((self.black & self.white).is_empty());
        if self.black.contains(&pos) {
            Some(Color::Black)
        } else if self.white.contains(&pos) {
            Some(Color::White)
        } else {
            None
        }
    }

    pub fn set(&mut self, pos: Pos, color: Color) {
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
            None => (Board::SIZE * Board::SIZE) as u32 - self.white.count() - self.black.count(),
        }
    }

    pub fn reverse(&self) -> Self {
        Self {
            black: self.white,
            white: self.black,
        }
    }

    pub fn flipped(&self, color: Color, pos: Pos) -> (usize, Self) {
        if (self.black | self.white).contains(&pos) {
            return (0, *self);
        }

        let mut res = *self;
        let (self_set, other_set) = match color {
            Color::Black => (&mut res.black, &mut res.white),
            Color::White => (&mut res.white, &mut res.black),
        };
        let mut count = 0;
        let mut flip_set = PosSet::new();
        for &points in pos.flip_lines() {
            let (c, s) = line_flipped(self_set, other_set, points);
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

    pub fn all_flipped(&self, color: Color) -> impl Iterator<Item = (Pos, Board)> + '_ {
        let other_set = match color {
            Color::Black => self.white,
            Color::White => self.black,
        };
        let candidates = !(self.black | self.white) & other_set.neighbors();
        candidates.into_iter().filter_map(move |pos| {
            let (cnt, board) = self.flipped(color, pos);
            (cnt > 0).then(|| (pos, board))
        })
    }

    pub fn can_flip(&self, color: Color, pos: Pos) -> bool {
        if (self.black | self.white).contains(&pos) {
            return false;
        }

        let (self_set, other_set) = match color {
            Color::Black => (self.black, self.white),
            Color::White => (self.white, self.black),
        };

        pos.flip_lines().iter().any(|points| {
            let (c, _m) = line_flipped(&self_set, &other_set, points);
            c > 0
        })
    }

    pub fn flip_candidates(&self, color: Color) -> impl Iterator<Item = Pos> + '_ {
        let other_set = match color {
            Color::Black => self.white,
            Color::White => self.black,
        };
        let candidates = !(self.black | self.white) & other_set.neighbors();
        candidates
            .into_iter()
            .filter(move |pos| self.can_flip(color, *pos))
    }

    pub fn can_play(&self, color: Color) -> bool {
        self.flip_candidates(color).next().is_some()
    }

    pub fn game_over(&self) -> bool {
        self.count(None) == 0
            || self.count(Some(Color::Black)) == 0
            || self.count(Some(Color::White)) == 0
            || (!self.can_play(Color::Black) && !self.can_play(Color::White))
    }

    pub fn from_pattern_index(pattern: &[Pos], index: u16) -> Self {
        let mut board = Self::empty();
        let mut n = index;
        for &pos in pattern {
            match n % 3 {
                0 => {}
                1 => board.set(pos, Color::Black),
                2 => board.set(pos, Color::White),
                _ => unreachable!(),
            }
            n /= 3;
            if n == 0 {
                break;
            }
        }
        debug_assert_eq!(board.pattern_index(pattern), index);
        board
    }

    pub fn pattern_index(&self, pattern: &[Pos]) -> u16 {
        let mut n = 0;
        let mut bit = 1;
        for pos in pattern {
            n += bit
                * match self.get(*pos) {
                    None => 0,
                    Some(Color::Black) => 1,
                    Some(Color::White) => 2,
                };
            bit *= 3;
        }
        n
    }
}

fn line_flipped(self_set: &PosSet, other_set: &PosSet, points: &[Pos]) -> (usize, PosSet) {
    let mut flipped = PosSet::new();
    for (count, pos) in points.iter().copied().enumerate() {
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
