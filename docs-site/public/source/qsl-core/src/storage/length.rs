//! Sealed physical length words. They never change a wire length/header.
mod sealed {
    pub trait Sealed {}
    impl Sealed for u32 {}
    impl Sealed for core::num::NonZeroU8 {}
    impl Sealed for core::num::NonZeroU16 {}
    impl Sealed for core::num::NonZeroU32 {}
}
pub trait LengthWord: sealed::Sealed + Copy {
    const EMPTY: Self;
    const MAX_LEN: usize;
    const SHIFT: bool;
    fn length(self) -> usize;
}
impl LengthWord for u32 {
    const EMPTY: Self = 0;
    const MAX_LEN: usize = u32::MAX as usize;
    const SHIFT: bool = false;
    fn length(self) -> usize {
        self as usize
    }
}
macro_rules! word {
    ($nz:ty,$int:ty) => {
        impl LengthWord for $nz {
            const EMPTY: Self = match Self::new(1) {
                Some(v) => v,
                None => panic!("one is nonzero"),
            };
            const MAX_LEN: usize = <$int>::MAX as usize - 1;
            const SHIFT: bool = true;
            fn length(self) -> usize {
                self.get() as usize - 1
            }
        }
    };
}
word!(core::num::NonZeroU8, u8);
word!(core::num::NonZeroU16, u16);
word!(core::num::NonZeroU32, u32);
/// Sealing guarantees that L is exactly an integer word, or its NonZero equivalent.
/// The bounds prove +1 cannot overflow and that a NonZero is never constructed from zero.
pub const fn length_word<L: LengthWord>(len: usize) -> L {
    assert!(len <= L::MAX_LEN);
    let encoded = len as u32 + L::SHIFT as u32;
    // Each source is the same size as L, has no padding, and has valid bits for L.
    unsafe {
        match core::mem::size_of::<L>() {
            1 => core::mem::transmute_copy(&(encoded as u8)),
            2 => core::mem::transmute_copy(&(encoded as u16)),
            4 => core::mem::transmute_copy(&encoded),
            _ => panic!("sealed word size"),
        }
    }
}

/// # Safety
/// len <= L::MAX_LEN. Kept separate so trusted pointer codecs have no length branch.
#[inline(always)]
pub unsafe fn length_word_unchecked<L: LengthWord>(len: usize) -> L {
    core::hint::assert_unchecked(len <= L::MAX_LEN);
    let encoded = len as u32 + L::SHIFT as u32;
    match core::mem::size_of::<L>() {
        1 => core::mem::transmute_copy(&(encoded as u8)),
        2 => core::mem::transmute_copy(&(encoded as u16)),
        4 => core::mem::transmute_copy(&encoded),
        _ => core::hint::unreachable_unchecked(),
    }
}
#[cfg(test)]
mod tests {
    use crate::{InlineString, StackString};
    use std::num::{NonZeroU16, NonZeroU8};
    #[test]
    fn shifted_storage_has_the_promised_bounds_niche_and_exact_wire() {
        const EMPTY: InlineString<8, NonZeroU8> = InlineString::new();
        const SPACES: InlineString<8, NonZeroU8> = InlineString::spaces::<8>();
        assert!(EMPTY.is_empty());
        assert_eq!(SPACES.as_str(), "        ");
        assert_eq!(std::mem::size_of::<StackString<254, NonZeroU8>>(), 255);
        assert_eq!(
            std::mem::size_of::<Option<StackString<254, NonZeroU8>>>(),
            255
        );
        assert_eq!(std::mem::size_of::<Option<InlineString<8, NonZeroU8>>>(), 9);
        for n in 0..=254 {
            let text = "a".repeat(n);
            let s = StackString::<254, NonZeroU8>::try_from_str(&text).unwrap();
            assert_eq!(s.len(), n);
            assert_eq!(s.as_str(), text);
            let a = unsafe { InlineString::<254>::from_str_unchecked(&text) };
            let b = unsafe { InlineString::<254, NonZeroU8>::from_str_unchecked(&text) };
            let (mut x, mut y) = ([0u8; 256], [0u8; 256]);
            let nx = unsafe { a.encode_to(x.as_mut_ptr()) };
            let ny = unsafe { b.encode_to(y.as_mut_ptr()) };
            assert_eq!(&x[..nx], &y[..ny]);
            let (d, used) = unsafe { InlineString::<254, NonZeroU8>::decode_from(y.as_ptr()) };
            assert_eq!(d.as_str(), text);
            assert_eq!(used, ny);
        }
        let s = StackString::<255, NonZeroU16>::try_from_str(&"b".repeat(255)).unwrap();
        assert_eq!(s.len(), 255);
        assert!(std::panic::catch_unwind(StackString::<255, NonZeroU8>::new).is_err());
    }
}
