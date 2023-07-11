#![doc = include_str!("../README.md")]

use core::mem::{size_of, align_of, ManuallyDrop, MaybeUninit};
use core::ptr::{addr_of, addr_of_mut};

/// Wraps a type in an opaque `#[repr(C)]` wrapper with a size equal to the original `T`
/// 
/// NOTE: The SIZE parameter expresses the number of `u64` required to contain `T`
#[repr(C)]
pub struct ReprCWrapper<const SIZE: usize, T> {
    buffer: MaybeUninit<[u64; SIZE]>,
    phantom: core::marker::PhantomData<T>,
}

impl<const SIZE: usize, T> From<T> for ReprCWrapper<SIZE, T> {
    fn from(val: T) -> Self {
        Self::new(val)
    }
}

impl<const SIZE: usize, T> Drop for ReprCWrapper<SIZE, T> {
    fn drop(&mut self) {
        unsafe{
            let val = &mut *self.buffer.as_mut_ptr().cast::<ManuallyDrop<T>>();
            ManuallyDrop::<T>::drop(val);
        }
    }
}

impl<const SIZE: usize, T> core::ops::Deref for ReprCWrapper<SIZE, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe{ &*addr_of!(self.buffer).cast::<ManuallyDrop::<T>>() }
    }
}

impl<const SIZE: usize, T> core::ops::DerefMut for ReprCWrapper<SIZE, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe{ &mut *addr_of_mut!(self.buffer).cast::<ManuallyDrop::<T>>() }
    }
}

impl<const SIZE: usize, T> ReprCWrapper<SIZE, T> {
    /// Returns a `ReprCWrapper` from a `T`
    pub fn new(val: T) -> Self {
        assert!(align_of::<T>() <= align_of::<Self>());
        assert_eq!(SIZE, (size_of::<ManuallyDrop::<T>>() + size_of::<u64>() - 1) / size_of::<u64>());

        let val = ManuallyDrop::<T>::new(val);
        let mut wrapper = Self {
            buffer: MaybeUninit::uninit(),
            phantom: core::marker::PhantomData
        };
        unsafe{ (wrapper.buffer.as_mut_ptr().cast::<ManuallyDrop::<T>>()).write(val); }
        wrapper
    }
    //NOTE: Use Deref & DerefMut instead of borrow() and borrow_mut()
    // Fewer methods for wrappers reduces the chance of interfering with T's methods
    // /// Borrows the inner `T` from a `ReprCWrapper`
    // pub fn borrow(&self) -> &T {
    //     unsafe{ &*addr_of!(self.buffer).cast::<ManuallyDrop::<T>>() }
    // }
    // /// Mutably borrows the inner `T` from a `ReprCWrapper`
    // pub fn borrow_mut(&mut self) -> &mut T {
    //     unsafe{ &mut *addr_of_mut!(self.buffer).cast::<ManuallyDrop::<T>>() }
    // }
    /// Consumes the `ReprCWrapper`, and returns the inner `T`
    pub fn into_inner(self) -> T {
        let val = unsafe{ core::ptr::read(addr_of!(self.buffer).cast::<ManuallyDrop<T>>()) };
        core::mem::forget(self);
        ManuallyDrop::<T>::into_inner(val)
    }
}

/// A `ReprCWrapper` type that corresponds to a wrapped version of `T`
///
/// NOTE: This macro is a stop-gap convenience to automatically get the correct type size,
/// until a future version of Rust stabilizes `generic_const_exprs`.  At that point, it will
/// be as simple as using `ReprCWrapper<T>`.
#[macro_export]
macro_rules! repr_c_wrapper_t {
    ( $t:ty ) => { $crate::ReprCWrapper<{(core::mem::size_of::<core::mem::ManuallyDrop::<$t>>() + core::mem::size_of::<u64>() - 1) / core::mem::size_of::<u64>()}, $t> };
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_basics() {

        let mut wrapper: repr_c_wrapper_t!(String) = "Hello".to_string().into();
        assert_eq!(*wrapper, "Hello");

        *wrapper = "World".to_string();
        assert_eq!(*wrapper, "World");

        let the_string = wrapper.into_inner();
        assert_eq!(the_string, "World");
    }

    struct VecCWrapper(repr_c_wrapper_t!(Vec<usize>));

    #[test]
    fn test_repr_c_wrapper_t_macro() {

        let wrapped_vec = VecCWrapper(vec![3, 2, 1].into());

        assert_eq!(*wrapped_vec.0, &[3, 2, 1]);
    }

    #[repr(align(8))]
    #[derive(Debug, Default, PartialEq)]
    struct WithPadding([u8;3]);
    struct WithPaddingCWrapper(repr_c_wrapper_t!(WithPadding));

    #[test]
    pub fn test_against_reading_uninit() {
        let wrapped = WithPaddingCWrapper(WithPadding::default().into());
        assert_eq!(&*wrapped.0, &WithPadding::default());
        let wrapped = WithPaddingCWrapper(WithPadding::default().into());
        assert_eq!(&*wrapped.0, &WithPadding::default());
    }

    #[repr(align(256))]
    #[derive(Debug, PartialEq)]
    struct WellAligned([u8;256]);
    struct WellAlignedCWrapper(repr_c_wrapper_t!(WellAligned));

    #[test]
    #[should_panic]
    pub fn test_against_unaligned() {
        let wrapped_wa = WellAlignedCWrapper(WellAligned([0; 256]).into());
        assert_eq!(&*wrapped_wa.0, &WellAligned([0; 256]));
        let wrapped_wa = WellAlignedCWrapper(WellAligned([0; 256]).into());
        assert_eq!(&*wrapped_wa.0, &WellAligned([0; 256]));
    }

}

//FUTURE NOTE: This implementation will get much cleaner when generic_const_exprs
// is merged into stable Rust.  https://github.com/rust-lang/rust/issues/76560
//When that happens, the trait and macros can be totally eliminated

// #![feature(generic_const_exprs)]

// #[repr(C)]
// pub struct ReprCWrapper<T>
//     where [(); (size_of::<ManuallyDrop::<T>>() + size_of::<u64>() - 1) / size_of::<u64>()]:
// {
//     bytes: [u64; (size_of::<ManuallyDrop::<T>>() + size_of::<u64>() - 1) / size_of::<u64>()],
// }
