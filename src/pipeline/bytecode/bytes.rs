use std::{mem, ops::BitOr};

#[inline(always)]
pub fn next_bytes<'l, const N: usize>(code: &mut impl Iterator<Item = &'l u8>) -> Option<[u8; N]> {
    let mut result = [0; N];
    for i in result.iter_mut() {
        *i = match code.next() {
            None => {
                return None;
            }
            Some(byte) => *byte,
        };
    }
    Some(result)
}

#[inline(always)]
pub fn try_next_bytes<const N: usize, Bytes, Extract>(
    code: &mut Bytes,
    extract: Extract,
) -> [Option<u8>; N]
where
    Bytes: Iterator,
    Extract: Fn(Bytes::Item) -> u8,
{
    let mut result = [None; N];
    for i in result.iter_mut() {
        *i = code.next().map(&extract);
    }
    result
}

pub trait FromBytes<Num> {
    fn bytes_to_num(&self) -> Num;
}

pub trait ToBytes<const SIZE: usize> {
    fn num_to_bytes(&self) -> [u8; SIZE];
}

impl FromBytes<u8> for [u8; 1] {
    #[inline(always)]
    fn bytes_to_num(&self) -> u8 {
        self[0]
    }
}

macro_rules! impl_FromBytes {
    ($num:ty, $size:expr) => {
        impl FromBytes<$num> for [u8; $size] {
            #[inline(always)]
            fn bytes_to_num(&self) -> $num {
                let mut result: $num = 0;
                for byte in self {
                    result = BitOr::<$num>::bitor(result << 8, From::from(*byte))
                }
                result
            }
        }
    };
}

macro_rules! impl_FromBytes_via {
    ($num:ty, $size:literal, $via_num:ty) => {
        impl FromBytes<$num> for [u8; $size] {
            #[inline(always)]
            fn bytes_to_num(&self) -> $num {
                <$num>::from(FromBytes::<$via_num>::bytes_to_num(self))
            }
        }
    };
}

impl_FromBytes_via!(u16, 1, u8);
impl_FromBytes!(u16, 2);
impl_FromBytes_via!(u32, 1, u8);
impl_FromBytes_via!(u32, 2, u16);
impl_FromBytes!(u32, 4);
impl_FromBytes_via!(u64, 1, u8);
impl_FromBytes_via!(u64, 2, u16);
impl_FromBytes_via!(u64, 4, u32);
impl_FromBytes!(u64, 8);
impl_FromBytes_via!(usize, 1, u8);
impl_FromBytes_via!(usize, 2, u16);
impl_FromBytes!(usize, mem::size_of::<usize>());

macro_rules! impl_ToBytes {
    ($size:expr, $num:ty) => {
        impl ToBytes<{ $size }> for $num {
            #[inline(always)]
            fn num_to_bytes(&self) -> [u8; $size] {
                let mut result: [u8; $size] = [0; $size];
                let bytes = self.to_be_bytes();
                (&mut result[$size - bytes.len()..]).copy_from_slice(&bytes);
                result
            }
        }
    };
}

impl_ToBytes!(1, u8);
impl_ToBytes!(2, u8);
impl_ToBytes!(4, u8);
impl_ToBytes!(8, u8);
impl_ToBytes!(2, u16);
impl_ToBytes!(4, u16);
impl_ToBytes!(8, u16);
impl_ToBytes!(4, u32);
impl_ToBytes!(8, u32);
impl_ToBytes!(8, u64);
impl_ToBytes!(mem::size_of::<usize>(), usize);

#[inline(always)]
pub fn all_there<const N: usize>(some_bytes: &[Option<u8>; N]) -> Option<[u8; N]> {
    let mut result = [0; N];
    for (index, place) in result.iter_mut().enumerate() {
        if let Some(byte) = some_bytes[index] {
            *place = byte;
        } else {
            return None;
        }
    }
    Some(result)
}

#[cfg(test)]
mod tests {
    use super::{FromBytes, ToBytes};

    #[test]
    fn round_trip() {
        assert_eq!(127u8, ToBytes::<1>::num_to_bytes(&127u8).bytes_to_num());
        assert_eq!(127u16, ToBytes::<2>::num_to_bytes(&127u8).bytes_to_num());
        assert_eq!(127u32, ToBytes::<4>::num_to_bytes(&127u8).bytes_to_num());
        assert_eq!(1234u16, ToBytes::<2>::num_to_bytes(&1234u16).bytes_to_num());
        assert_eq!(1234u32, ToBytes::<4>::num_to_bytes(&1234u16).bytes_to_num());
    }
}
