use tlist::{Len, TCons, TList, TNil};

pub struct Byte<const VAL: u8>;

/// Size of the internal buffer used to build up a `&'static str` from a `TypeStr`.
/// This is the larges allowed string literal that can be used with `FromLiteralStr`.
///
/// Currently defined as 32KiB.
///
/// The exact value needs to be fixed but is arbitrary.
/// If you really need a larger value, please open a PR.
// NOTE: Make sure this value is not smaller than the one in `overloaded_literals_macro`
pub const MAX_STR_LIT_LEN: usize = 32768;

pub trait TypeStr: TList {
    const LEN: usize;
    const V: [u8; MAX_STR_LIT_LEN];

    const STR: &'static str = {
        let ptr = &Self::V as *const u8;
        let slice = unsafe { core::slice::from_raw_parts::<'static, u8>(ptr, Self::LEN) };
        unsafe { core::str::from_utf8_unchecked(slice) }
    };
}

impl TypeStr for TNil {
    const LEN: usize = 0;
    const V: [u8; MAX_STR_LIT_LEN] = { [0; MAX_STR_LIT_LEN] };
}

pub trait ContainsByte {
    const BYTE: u8;
}
impl<const BYTE: u8> ContainsByte for Byte<BYTE> {
    const BYTE: u8 = BYTE;
}

use typenum::Unsigned;
impl<First: ContainsByte, Rest: TypeStr> TypeStr for TCons<First, Rest> {
    const LEN: usize = Len::<Self>::USIZE;
    const V: [u8; MAX_STR_LIT_LEN] = {
        assert!(Len::<Self>::USIZE <= MAX_STR_LIT_LEN);

        let first = First::BYTE;
        let first_elem_ptr = core::ptr::addr_of!(first);

        let rest = Rest::V;
        let rest_ptr = core::ptr::addr_of!(rest) as *const u8;

        // Conceptually, this is: `let mut arr = [0; MAX_STR_LIT_LEN]; let target_ptr = core::mem::addr_of_mut!(arr);`
        // NOTE: **needs** to be mut because we build a *mut u8 pointer from it below.
        // If it were not mut, we would trigger UB ('mutate data owned by an immutable binding').
        // because *conceptually* we do a mutable borrow here.
        // We emulate `addr_of_mut!` (which is blocked by the 'const_mut_refs' feature) using transmute.
        // This goes [u8; MAX_STR_LIT_LEN] -> *const [u8; MAX_STR_LIT_LEN] -> *const u8 -> *mut u8
        #[allow(unused_mut)]
        let mut arr: [u8; MAX_STR_LIT_LEN] = [0; MAX_STR_LIT_LEN];
        let target_ptr = unsafe { core::mem::transmute(core::ptr::addr_of!(arr) as *const u8) }; // <- Poor man's addr_of_mut!

        unsafe { core::ptr::copy_nonoverlapping(first_elem_ptr, target_ptr, 1) };
        unsafe { core::ptr::copy_nonoverlapping(rest_ptr, target_ptr.add(1), Len::<Rest>::USIZE) };
        arr
    };
}
