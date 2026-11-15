use crate::InlineArray;
use core::num::{NonZeroU16, NonZeroU8};
#[test]
fn nonzero_lengths_preserve_empty_values_and_option_niches() {
    type A = InlineArray<u8, 4, NonZeroU8>;
    assert_eq!(core::mem::size_of::<A>(), 5);
    assert_eq!(core::mem::size_of::<Option<A>>(), core::mem::size_of::<A>());
    let mut a = A::new();
    assert!(a.is_empty());
    for i in 0..4 {
        a.push(i).unwrap();
    }
    assert!(a.push(9).is_err());
    assert_eq!(a.as_slice(), [0, 1, 2, 3]);
    assert_eq!(a.clone().into_array().unwrap(), [0, 1, 2, 3]);
    a.clear();
    assert!(a.is_empty());
    a.push(42).unwrap();
    assert_eq!(a.as_slice(), [42]);
    let mut b = InlineArray::<u16, 255, NonZeroU16>::new();
    b.push(65535).unwrap();
    assert_eq!(b.as_slice(), [65535]);
}
#[test]
fn drop_visits_only_the_initialized_prefix() {
    use std::{cell::Cell, rc::Rc};
    struct D(Rc<Cell<usize>>);
    impl Drop for D {
        fn drop(&mut self) {
            self.0.set(self.0.get() + 1);
        }
    }
    let count = Rc::new(Cell::new(0));
    {
        let mut a = InlineArray::<D, 4, NonZeroU8>::new();
        a.push(D(count.clone())).unwrap();
        a.push(D(count.clone())).unwrap();
        a.clear();
        assert_eq!(count.get(), 2);
        a.push(D(count.clone())).unwrap();
    }
    assert_eq!(count.get(), 3);
}
#[test]
fn native_copy_initializes_the_selected_length_word() {
    let source = [1u16, 65535, 3];
    let mut out = core::mem::MaybeUninit::<InlineArray<u16, 3, NonZeroU8>>::uninit();
    unsafe {
        InlineArray::copy_native_into(source.as_ptr().cast(), 3, out.as_mut_ptr());
        assert_eq!(out.assume_init().as_slice(), source);
    }
}
#[test]
fn insufficient_nonzero_width_is_rejected_before_use() {
    assert!(std::panic::catch_unwind(InlineArray::<u8, 255, NonZeroU8>::new).is_err());
}
