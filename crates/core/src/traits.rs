pub trait IterOnes: Sized {
    fn iter_ones(&self) -> Ones<Self>;
}

#[derive(Debug, Clone, Copy)]
pub struct Ones<T>(T);

macro_rules! impl_ones {
    ($($ty:ident),* $(,)?) => {
        $(
            impl IterOnes for $ty {
                fn iter_ones(&self) -> Ones<Self> {
                    Ones(*self)
                }
            }

            impl Iterator for Ones<$ty> {
                type Item = u32;

                fn next(&mut self) -> Option<Self::Item> {
                    if self.0 == 0 {
                        return None;
                    }
                    let idx = self.0.trailing_zeros();
                    let bit = 1 << idx;
                    self.0 &= !bit;
                    Some(idx)
                }

                fn size_hint(&self) -> (usize, Option<usize>) {
                    let count = self.0.count_ones() as usize;
                    (count, Some(count))
                }
            }

            impl DoubleEndedIterator for Ones<$ty> {
                fn next_back(&mut self) -> Option<Self::Item> {
                    if self.0 == 0 {
                        return None;
                    }
                    let idx = $ty::BITS - self.0.leading_zeros() - 1;
                    let bit = 1 << idx;
                    self.0 &= !bit;
                    Some(idx)
                }
            }

            impl ExactSizeIterator for Ones<$ty> {}
        )*
    };
}
impl_ones!(u8, u16, u32, u64, u128);

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn iter_ones() {
        (0b10101u8).iter_ones().eq([0, 2, 4]);
        (0b10101u8).iter_ones().rev().eq([4, 2, 0]);
        0u8.iter_ones().eq([]);
        0u8.iter_ones().rev().eq([]);
    }
}
