use std::num::NonZeroUsize;

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

    pub fn flipped(&self, pos: Pos) -> Option<(NonZeroUsize, Self)> {
        if (self.mine_disks | self.others_disks).contains(&pos) {
            return None;
        }

        let (count, flipped) = pos
            .flip_lines()
            .flipped(&self.mine_disks, &self.others_disks);
        let count = NonZeroUsize::new(count)?;
        let board = Board {
            mine_disks: self.others_disks & !flipped,
            others_disks: self.mine_disks | flipped,
        };
        Some((count, board))
    }

    pub fn passed(&self) -> Self {
        Board {
            mine_disks: self.others_disks,
            others_disks: self.mine_disks,
        }
    }

    pub fn all_flipped(&self) -> impl Iterator<Item = (Pos, Board)> + '_ {
        let candidates = !(self.mine_disks | self.others_disks) & self.others_disks.neighbors();
        candidates
            .into_iter()
            .filter_map(move |pos| self.flipped(pos).map(|(_count, board)| (pos, board)))
    }

    pub fn can_flip(&self, pos: Pos) -> bool {
        if (self.mine_disks | self.others_disks).contains(&pos) {
            return false;
        }
        pos.flip_lines()
            .can_flip(&self.mine_disks, &self.others_disks)
    }

    pub fn flip_candidates(&self) -> impl Iterator<Item = Pos> + '_ {
        let candidates = !(self.mine_disks | self.others_disks) & self.others_disks.neighbors();
        candidates
            .into_iter()
            .filter(move |pos| self.can_flip(*pos))
    }

    pub fn can_play(&self) -> bool {
        self.flip_candidates().next().is_some()
    }

    pub fn game_over(&self) -> bool {
        self.count_disk(None) == 0
            || self.count_disk(Some(Disk::Mine)) == 0
            || self.count_disk(Some(Disk::Others)) == 0
            || (!self.can_play() && !self.passed().can_play())
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

    #[test]
    fn flip_candidates() {
        use Pos as P;

        let board = Board::new();
        assert!(board.flip_candidates().eq([P::C4, P::D3, P::E6, P::F5]));

        assert_eq!(board.flipped(P::A1), None);
        let (count, board) = board.flipped(P::D3).unwrap();
        assert_eq!(count.get(), 2);
        assert!(board.mine_disks.into_iter().eq([P::E5]));
        assert!(board
            .others_disks
            .into_iter()
            .eq([P::D3, P::D4, P::D5, P::E4]));

        assert!(board.flip_candidates().eq([P::C3, P::C5, P::E3]));
        let (count, board) = board.flipped(P::C5).unwrap();
        assert_eq!(count.get(), 2);
        assert!(board.mine_disks.into_iter().eq([P::D3, P::D4, P::E4]));
        assert!(board.others_disks.into_iter().eq([P::C5, P::D5, P::E5]));

        assert!(board
            .flip_candidates()
            .eq([P::B6, P::C6, P::D6, P::E6, P::F6]));
    }

    #[test]
    fn pass() {
        use Pos as P;
        let mut board = Board::new();
        let hands = [P::D3, P::C3, P::F5, P::D2, P::D1, P::E1, P::B2, P::C1];

        for hand in hands {
            board = board.flipped(hand).unwrap().1;
        }
        assert!(!board.can_play());
        board = board.passed();
        assert!(board.can_play());
    }
}
