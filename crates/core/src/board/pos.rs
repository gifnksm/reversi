use super::{Board, Direction};
use crate::traits::{IterOnes, Ones};
use std::{
    fmt,
    iter::{self, FromIterator},
    num::ParseIntError,
    str::FromStr,
};

const AVAILABLE_BITS: u128 = 0b0111111110 << (Board::SIZE + 2)
    | 0b0111111110 << (2 * (Board::SIZE + 2))
    | 0b0111111110 << (3 * (Board::SIZE + 2))
    | 0b0111111110 << (4 * (Board::SIZE + 2))
    | 0b0111111110 << (5 * (Board::SIZE + 2))
    | 0b0111111110 << (6 * (Board::SIZE + 2))
    | 0b0111111110 << (7 * (Board::SIZE + 2))
    | 0b0111111110 << (8 * (Board::SIZE + 2));

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Pos(i32);

impl fmt::Debug for Pos {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}({})", self, self.0)
    }
}

impl fmt::Display for Pos {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let col = match self.x() {
            0 => 'A',
            1 => 'B',
            2 => 'C',
            3 => 'D',
            4 => 'E',
            5 => 'F',
            6 => 'G',
            7 => 'H',
            _ => unreachable!(),
        };
        let row = self.y() + 1;
        write!(f, "{}{}", col, row)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ParsePosError {
    #[error("cannot parse pos from empty string")]
    Empty,
    #[error("invalid alphabet `{0}` found in string")]
    InvalidAlphabet(char),
    #[error("cannot parse `{0}` as number: `{1}`")]
    ParseInt(String, ParseIntError),
    #[error("invalid pos `{0}{1}`")]
    InvalidPos(char, i32),
}

impl FromStr for Pos {
    type Err = ParsePosError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.to_ascii_uppercase();
        let mut cs = s.chars();

        let alpha = cs.next().ok_or(Self::Err::Empty).and_then(|alpha| {
            if !alpha.is_alphabetic() {
                return Err(Self::Err::InvalidAlphabet(alpha));
            }
            Ok(alpha)
        })?;
        let num = cs
            .as_str()
            .parse::<i32>()
            .map_err(|e| ParsePosError::ParseInt(cs.as_str().into(), e))?;

        let x = (alpha as u8 - b'A') as i32;
        let y = num - 1;
        Pos::from_xy(x, y).ok_or_else(|| Self::Err::InvalidPos(alpha, num))
    }
}

macro_rules! define_pos {
    ($($name:ident: ($x:expr, $y:expr)),* $(,)?) => {
        $(
            #[allow(dead_code)]
            pub const $name: Self = match Self::from_xy($x, $y) {
                Some(pos) => pos,
                None => loop {},
            };
        )*
    };
}

impl Pos {
    define_pos! {
        A1: (0, 0), A2: (0, 1), A3: (0, 2), A4: (0, 3),
        A5: (0, 4), A6: (0, 5), A7: (0, 6), A8: (0, 7),
        B1: (1, 0), B2: (1, 1), B3: (1, 2), B4: (1, 3),
        B5: (1, 4), B6: (1, 5), B7: (1, 6), B8: (1, 7),
        C1: (2, 0), C2: (2, 1), C3: (2, 2), C4: (2, 3),
        C5: (2, 4), C6: (2, 5), C7: (2, 6), C8: (2, 7),
        D1: (3, 0), D2: (3, 1), D3: (3, 2), D4: (3, 3),
        D5: (3, 4), D6: (3, 5), D7: (3, 6), D8: (3, 7),
        E1: (4, 0), E2: (4, 1), E3: (4, 2), E4: (4, 3),
        E5: (4, 4), E6: (4, 5), E7: (4, 6), E8: (4, 7),
        F1: (5, 0), F2: (5, 1), F3: (5, 2), F4: (5, 3),
        F5: (5, 4), F6: (5, 5), F7: (5, 6), F8: (5, 7),
        G1: (6, 0), G2: (6, 1), G3: (6, 2), G4: (6, 3),
        G5: (6, 4), G6: (6, 5), G7: (6, 6), G8: (6, 7),
        H1: (7, 0), H2: (7, 1), H3: (7, 2), H4: (7, 3),
        H5: (7, 4), H6: (7, 5), H7: (7, 6), H8: (7, 7),
    }

    pub const fn from_xy(x: i32, y: i32) -> Option<Self> {
        if 0 <= x && x < Board::SIZE && 0 <= y && y < Board::SIZE {
            Some(Self((x + 1) * (Board::SIZE + 2) + (y + 1)))
        } else {
            None
        }
    }

    pub const fn from_index(index: i32) -> Option<Self> {
        if (1 << index) & AVAILABLE_BITS != 0 {
            Some(Self(index))
        } else {
            None
        }
    }

    const fn bit(&self) -> PosSet {
        PosSet(1 << self.0)
    }

    pub const fn x(&self) -> i32 {
        self.0 / (Board::SIZE + 2) - 1
    }

    pub const fn y(&self) -> i32 {
        self.0 % (Board::SIZE + 2) - 1
    }

    pub fn line(&self, dir: Direction) -> impl Iterator<Item = Self> {
        let amt = dir.to_add_amount();
        iter::successors(Some(*self), move |p| Self::from_index(p.0 + amt)).skip(1)
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PosSet(u128);

impl PosSet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_empty(&self) -> bool {
        self.0 == 0
    }

    pub fn count(&self) -> u32 {
        self.0.count_ones()
    }

    pub fn contains(&self, pos: &Pos) -> bool {
        self.0 & pos.bit().0 != 0
    }

    pub fn neighbors(&self) -> PosSet {
        let mut neighbor_bits = 0;
        for dir in Direction::ALL {
            let amt = dir.to_add_amount();
            if amt < 0 {
                neighbor_bits |= self.0 >> (-amt);
            } else {
                neighbor_bits |= self.0 << amt;
            }
        }
        Self(neighbor_bits & AVAILABLE_BITS)
    }
}

impl std::ops::Not for PosSet {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self(!self.0 & AVAILABLE_BITS)
    }
}

macro_rules! impl_bit_ops {
    ($trait:ident, $name:ident, $op:tt) => {
        impl std::ops::$trait<Pos> for Pos {
            type Output = PosSet;

            fn $name(self, rhs: Pos) -> Self::Output {
                self.bit() $op rhs.bit()
            }
        }

        impl std::ops::$trait<Pos> for PosSet {
            type Output = Self;

            fn $name(self, rhs: Pos) -> Self::Output {
                self $op rhs.bit()
            }
        }

        impl std::ops::$trait<PosSet> for PosSet {
            type Output = Self;

            fn $name(self, rhs: PosSet) -> Self::Output {
                Self(self.0 $op rhs.0)
            }
        }
    };
}
impl_bit_ops!(BitOr, bitor, |);
impl_bit_ops!(BitAnd, bitand, &);
impl_bit_ops!(BitXor, bitxor, ^);

macro_rules! impl_bit_assign_ops {
    ($trait:ident, $name:ident, $op:tt) => {
        impl std::ops::$trait<Pos> for PosSet {
            fn $name(&mut self, rhs: Pos) {
                *self $op rhs.bit();
            }
        }

        impl std::ops::$trait<PosSet> for PosSet {
            fn $name(&mut self, rhs: PosSet) {
                self.0 $op rhs.0;
            }
        }
    };
}
impl_bit_assign_ops!(BitOrAssign, bitor_assign, |=);
impl_bit_assign_ops!(BitAndAssign, bitand_assign, &=);
impl_bit_assign_ops!(BitXorAssign, bitxor_assign, ^=);

impl IntoIterator for PosSet {
    type Item = Pos;
    type IntoIter = PosSetIter;

    fn into_iter(self) -> Self::IntoIter {
        PosSetIter(self.0.iter_ones())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PosSetIter(Ones<u128>);

impl Iterator for PosSetIter {
    type Item = Pos;

    fn next(&mut self) -> Option<Self::Item> {
        let idx = self.0.next()?;
        Pos::from_index(idx as i32)
    }
}

impl FromIterator<Pos> for PosSet {
    fn from_iter<T: IntoIterator<Item = Pos>>(iter: T) -> Self {
        let mut set = Self::new();
        for pos in iter {
            set |= pos;
        }
        set
    }
}

impl Direction {
    fn to_add_amount(self) -> i32 {
        let up = -1;
        let down = 1;
        let left = -(Board::SIZE + 2);
        let right = Board::SIZE + 2;
        match self {
            Self::UpLeft => up + left,
            Self::Up => up,
            Self::UpRight => up + right,
            Self::Left => left,
            Self::Right => right,
            Self::DownLeft => down + left,
            Self::Down => down,
            Self::DownRight => down + right,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn available_bits() {
        let mut ava_bits = PosSet::new();
        for y in 0..Board::SIZE {
            for x in 0..Board::SIZE {
                ava_bits |= Pos::from_xy(x, y).unwrap().bit();
            }
        }
        assert_eq!(ava_bits.0, AVAILABLE_BITS);
    }

    #[test]
    fn display() {
        assert_eq!(Pos::A1.to_string(), "A1");
        assert_eq!(Pos::C8.to_string(), "C8");
        assert_eq!(Pos::H3.to_string(), "H3");
    }

    #[test]
    fn from_str() {
        assert_eq!(Pos::from_str("A1").unwrap(), Pos::A1);
        assert_eq!(Pos::from_str("C8").unwrap(), Pos::C8);
        assert_eq!(Pos::from_str("H3").unwrap(), Pos::H3);
    }

    #[test]
    fn get_xy() {
        for y in 0..Board::SIZE {
            for x in 0..Board::SIZE {
                let p = Pos::from_xy(x, y).unwrap();
                assert_eq!(p.x(), x);
                assert_eq!(p.y(), y);
            }
        }
        fn to_xy(pos: Pos) -> (i32, i32) {
            (pos.x(), pos.y())
        }
        assert_eq!(to_xy(Pos::A1), (0, 0));
        assert_eq!(to_xy(Pos::D4), (3, 3));
        assert_eq!(to_xy(Pos::H8), (7, 7));
    }

    #[test]
    fn ord() {
        let sorted = [Pos::A1, Pos::A2, Pos::A3, Pos::B3, Pos::C1];
        let mut cloned = sorted;
        cloned.sort();
        assert_eq!(sorted, cloned);
    }

    #[test]
    fn line() {
        use Direction::*;
        use Pos as P;

        let origin = P::D5;
        let lines: &[(_, &[_])] = &[
            (UpLeft, &[P::C4, P::B3, P::A2]),
            (Up, &[P::D4, P::D3, P::D2, P::D1]),
            (UpRight, &[P::E4, P::F3, P::G2, P::H1]),
            (Left, &[P::C5, P::B5, P::A5]),
            (Right, &[P::E5, P::F5, P::G5, P::H5]),
            (DownLeft, &[P::C6, P::B7, P::A8]),
            (Down, &[P::D6, P::D7, P::D8]),
            (DownRight, &[P::E6, P::F7, P::G8]),
        ];

        for (dir, pos) in lines {
            assert!(origin.line(*dir).eq(pos.iter().copied()));
        }
    }
}
