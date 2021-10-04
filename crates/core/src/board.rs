pub use self::{color::*, pos::*};

mod color;
mod pos;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Board {
    mine_disks: PosSet,
    others_disks: PosSet,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Disk {
    Mine,
    Others,
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
            mine_disks: PosSet::new() | Pos::E4 | Pos::D5,
            others_disks: PosSet::new() | Pos::D4 | Pos::E5,
        }
    }

    pub fn empty() -> Self {
        Self {
            mine_disks: PosSet::new(),
            others_disks: PosSet::new(),
        }
    }

    pub fn get_disk(&self, pos: Pos) -> Option<Disk> {
        debug_assert!((self.mine_disks & self.others_disks).is_empty());
        if self.mine_disks.contains(&pos) {
            Some(Disk::Mine)
        } else if self.others_disks.contains(&pos) {
            Some(Disk::Others)
        } else {
            None
        }
    }

    pub fn set_disk(&mut self, pos: Pos, disk: Disk) {
        let (insert_mask, remove_mask) = match disk {
            Disk::Mine => (&mut self.mine_disks, &mut self.others_disks),
            Disk::Others => (&mut self.others_disks, &mut self.mine_disks),
        };
        let mask = PosSet::new() | pos;
        *insert_mask |= mask;
        *remove_mask &= !mask;
    }

    pub fn unset_disk(&mut self, pos: Pos) {
        let mask = PosSet::new() | pos;
        self.mine_disks &= !mask;
        self.others_disks &= !mask;
    }

    pub fn count_disk(&self, disk: Option<Disk>) -> u32 {
        match disk {
            Some(Disk::Mine) => self.mine_disks.count(),
            Some(Disk::Others) => self.others_disks.count(),
            None => {
                (Board::SIZE * Board::SIZE) as u32
                    - self.mine_disks.count()
                    - self.others_disks.count()
            }
        }
    }

    pub fn count_all_disks(&self) -> u32 {
        self.mine_disks.count() + self.others_disks.count()
    }

    pub fn reverse(&self) -> Self {
        Board {
            mine_disks: self.others_disks,
            others_disks: self.mine_disks,
        }
    }

    fn flipped_set(&self, pos: Pos) -> PosSet {
        debug_assert!(!(self.mine_disks | self.others_disks).contains(&pos));
        let top_bottom_mask = PosSet::ALL;
        let left_right_mask = !(PosSet::new()
            | (Pos::A1 | Pos::A2 | Pos::A3 | Pos::A4 | Pos::A5 | Pos::A6 | Pos::A7 | Pos::A8)
            | (Pos::H1 | Pos::H2 | Pos::H3 | Pos::H4 | Pos::H5 | Pos::H6 | Pos::H7 | Pos::H8));
        let pos = PosSet::new() | pos;

        let right_moves = |mask, offset| {
            let e = self.others_disks & mask;
            let mut m = (pos << offset) & e;
            m |= (m << offset) & e;
            m |= (m << offset) & e;
            m |= (m << offset) & e;
            m |= (m << offset) & e;
            m |= (m << offset) & e;
            let mut o = (self.mine_disks >> offset) & e;
            o |= (o >> offset) & e;
            o |= (o >> offset) & e;
            o |= (o >> offset) & e;
            o |= (o >> offset) & e;
            o |= (o >> offset) & e;
            m & o
        };

        let left_moves = |mask, offset| {
            let e = self.others_disks & mask;
            let mut m = (pos >> offset) & e;
            m |= (m >> offset) & e;
            m |= (m >> offset) & e;
            m |= (m >> offset) & e;
            m |= (m >> offset) & e;
            m |= (m >> offset) & e;
            let mut o = (self.mine_disks << offset) & e;
            o |= (o << offset) & e;
            o |= (o << offset) & e;
            o |= (o << offset) & e;
            o |= (o << offset) & e;
            o |= (o << offset) & e;
            m & o
        };

        let flipped = left_moves(left_right_mask, 1)
            | left_moves(left_right_mask, 9)
            | left_moves(top_bottom_mask, 8)
            | left_moves(left_right_mask, 7)
            | right_moves(left_right_mask, 1)
            | right_moves(left_right_mask, 9)
            | right_moves(top_bottom_mask, 8)
            | right_moves(left_right_mask, 7);

        debug_assert!((self.mine_disks & flipped).is_empty());
        debug_assert_eq!(self.others_disks & flipped, flipped);

        flipped
    }

    pub fn flipped(&self, pos: Pos) -> Option<Self> {
        if (self.mine_disks | self.others_disks).contains(&pos) {
            return None;
        }

        let flipped = self.flipped_set(pos);
        (!flipped.is_empty()).then(|| Board {
            mine_disks: self.others_disks ^ flipped,
            others_disks: self.mine_disks ^ flipped ^ pos,
        })
    }

    pub fn all_flipped(&self) -> impl Iterator<Item = (Pos, Board)> + '_ {
        self.flip_candidates().into_iter().map(move |pos| {
            let flipped = self.flipped_set(pos);
            let board = Self {
                mine_disks: self.others_disks ^ flipped,
                others_disks: self.mine_disks ^ flipped ^ pos,
            };
            (pos, board)
        })
    }

    pub fn can_flip(&self, pos: Pos) -> bool {
        !(self.mine_disks | self.others_disks).contains(&pos) && !self.flipped_set(pos).is_empty()
    }

    pub fn flip_candidates(&self) -> PosSet {
        let top_bottom_mask = PosSet::ALL;
        let left_right_mask = !(PosSet::new()
            | (Pos::A1 | Pos::A2 | Pos::A3 | Pos::A4 | Pos::A5 | Pos::A6 | Pos::A7 | Pos::A8)
            | (Pos::H1 | Pos::H2 | Pos::H3 | Pos::H4 | Pos::H5 | Pos::H6 | Pos::H7 | Pos::H8));
        let empty_cells = !self.mine_disks & !self.others_disks;

        let right_moves = |mask, offset| {
            let e = self.others_disks & mask;
            let mut m = (self.mine_disks << offset) & e;
            m |= (m << offset) & e;
            m |= (m << offset) & e;
            m |= (m << offset) & e;
            m |= (m << offset) & e;
            m |= (m << offset) & e;
            m << offset
        };

        let left_moves = |mask, offset| {
            let e = self.others_disks & mask;
            let mut m = (self.mine_disks >> offset) & e;
            m |= (m >> offset) & e;
            m |= (m >> offset) & e;
            m |= (m >> offset) & e;
            m |= (m >> offset) & e;
            m |= (m >> offset) & e;
            m >> offset
        };

        empty_cells
            & (left_moves(left_right_mask, 1)
                | left_moves(left_right_mask, 9)
                | left_moves(top_bottom_mask, 8)
                | left_moves(left_right_mask, 7)
                | right_moves(left_right_mask, 1)
                | right_moves(left_right_mask, 9)
                | right_moves(top_bottom_mask, 8)
                | right_moves(left_right_mask, 7))
    }

    pub fn can_play(&self) -> bool {
        !self.flip_candidates().is_empty()
    }

    pub fn from_pattern_index(pattern: &[Pos], index: u16) -> Self {
        let mut board = Self::empty();
        let mut n = index;
        for &pos in pattern {
            match n % 3 {
                0 => {}
                1 => board.set_disk(pos, Disk::Mine),
                2 => board.set_disk(pos, Disk::Others),
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
                * match self.get_disk(*pos) {
                    None => 0,
                    Some(Disk::Mine) => 1,
                    Some(Disk::Others) => 2,
                };
            bit *= 3;
        }
        n
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::iter::FromIterator;

    #[test]
    fn flip_candidates() {
        use Pos as P;

        let board = Board::new();
        assert_eq!(
            board.flip_candidates(),
            PosSet::from_iter([P::D3, P::C4, P::F5, P::E6])
        );

        assert_eq!(board.flipped(P::A1), None);
        let board = board.flipped(P::D3).unwrap();
        assert_eq!(board.mine_disks, PosSet::from_iter([P::E5]));
        assert_eq!(
            board.others_disks,
            PosSet::from_iter([P::D3, P::D4, P::E4, P::D5])
        );

        assert_eq!(
            board.flip_candidates(),
            PosSet::from_iter([P::C3, P::E3, P::C5])
        );
        let board = board.flipped(P::C5).unwrap();
        assert_eq!(board.mine_disks, PosSet::from_iter([P::D3, P::D4, P::E4]));
        assert_eq!(board.others_disks, PosSet::from_iter([P::C5, P::D5, P::E5]));

        assert_eq!(
            board.flip_candidates(),
            PosSet::from_iter([P::B6, P::C6, P::D6, P::E6, P::F6])
        );
    }

    #[test]
    fn pass() {
        use Pos as P;
        let mut board = Board::new();
        let hands = [P::D3, P::C3, P::F5, P::D2, P::D1, P::E1, P::B2, P::C1];

        for hand in hands {
            board = board.flipped(hand).unwrap();
        }
        assert!(!board.can_play());
        board = board.reverse();
        assert!(board.can_play());
    }
}
