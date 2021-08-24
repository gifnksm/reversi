pub trait IterOnes: Sized {
    fn iter_ones(&self) -> Ones<Self>;
}

impl IterOnes for u128 {
    fn iter_ones(&self) -> Ones<Self> {
        Ones(*self)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Ones<T>(T);

macro_rules! impl_ones {
    ($($ty:ty),* $(,)?) => {
        $(
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
            }
        )*
    };
}
impl_ones!(u8, u16, u32, u64, u128);

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn iter_ones() {
        (0b10101).iter_ones().eq([0, 2, 4]);
        0.iter_ones().eq([]);
    }
}
