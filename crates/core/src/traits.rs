use std::iter::FusedIterator;

pub trait IterOneBits: Sized {
    fn iter_one_bits(&self) -> OneBits<Self>;
}

#[derive(Debug, Clone, Copy)]
pub struct OneBits<T>(T);

macro_rules! impl_ones {
    ($($ty:ident),* $(,)?) => {
        $(
            impl IterOneBits for $ty {
                fn iter_one_bits(&self) -> OneBits<Self> {
                    OneBits(*self)
                }
            }

            impl Iterator for OneBits<$ty> {
                type Item = $ty;

                fn next(&mut self) -> Option<Self::Item> {
                    if self.0 == 0 {
                        return None;
                    }
                    let idx = self.0.trailing_zeros();
                    let bit = 1 << idx;
                    self.0 &= !bit;
                    Some(bit)
                }

                fn size_hint(&self) -> (usize, Option<usize>) {
                    let count = self.0.count_ones() as usize;
                    (count, Some(count))
                }
            }

            impl DoubleEndedIterator for OneBits<$ty> {
                fn next_back(&mut self) -> Option<Self::Item> {
                    if self.0 == 0 {
                        return None;
                    }
                    let idx = $ty::BITS - self.0.leading_zeros() - 1;
                    let bit = 1 << idx;
                    self.0 &= !bit;
                    Some(bit)
                }
            }

            impl ExactSizeIterator for OneBits<$ty> {}
            impl FusedIterator for OneBits<$ty> {}
        )*
    };
}
impl_ones!(u8, u16, u32, u64, u128);

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn iter_ones() {
        (0b10101u8).iter_one_bits().eq([1, 1 << 2, 1 << 4]);
        (0b10101u8).iter_one_bits().rev().eq([1 << 4, 1 << 2, 0]);
        0u8.iter_one_bits().eq([]);
        0u8.iter_one_bits().rev().eq([]);
    }
}
