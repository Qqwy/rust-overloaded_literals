//! Lifting of a str literal to the type level,
//! to allow usage of a `const &'static str` in generic const contexts
//! (which is otherwise not allowed on stable Rust currently.)
//!
//! Items in this module need to be public as the types and structs contained within
//! are built by the `overloaded_literals!` macro.
//!
//! However, consider the contents of this module an implementation detail,
//! and do not depend on these details directly in your code. They are subject to change.
//!
//! The only API which is guaranteed, is that the `TStr` trait will expose an associated const value with the signature `const STR: &'static str;`.
use tlist::{Len, TCons, TList, TNil};
use typenum::Unsigned;

/// Struct to lift a single u8 byte to the type level.
///
/// Implementation detail of [TypeStr].
pub struct Byte<const VAL: u8>;

/// Implementation detail of [Byte] to read out its contained value in a generic context
pub trait ContainsByte {
    const BYTE: u8;
}

impl<const BYTE: u8> ContainsByte for Byte<BYTE> {
    const BYTE: u8 = BYTE;
}

/// Size of the internal buffer used to build up a `&'static str` from a `TypeStr`.
/// This is the larges allowed string literal that can be used with `FromLiteralStr`.
///
/// Currently defined as 32KiB.
///
/// The exact value needs to be fixed but is arbitrary.
/// If you really need a larger value, please open a PR.
// NOTE: Make sure this value is not smaller than the one in `overloaded_literals_macro`
pub const MAX_STR_LIT_LEN: usize = 32768;

/// Trait to work with [str]s at the type level, allowing &'static str in 'const generic' contexts.
///
/// # How it works
/// _This information is not important for normal use of the library. It is only interesting if you want to work on the internals of the library, or are curious._
///
/// Each string literal compiles down (using the macro) to its own type-level list (c.f. [trait@TList]),
/// in which each of the elements in the list is a [Byte].
/// These disparate [Byte] types are manipulated in a generic way
/// because regardless of their content VAL, they all implement the [ContainsByte] trait.
///
/// So for a string like
/// ```rust
/// "hello"
/// ```
/// the macro builds the type-level list:
/// ```rust
/// TList![Byte<104>, Byte<101>, Byte<108>, Byte<108>, Byte<111>]
/// // Which is syntactic sugar for:
/// TCons<Byte<104>, TCons<Byte<101>, TCons<Byte<108>, TCons<Byte<108>, TCons<Byte<111>, TNil>>>>>
/// ```
///
/// The two implementations of the [TypeStr] trait (for [TNil] and [TCons]) then join together these u8 values
/// from each of the elements in the type-level linked list,
/// putting them in a large const array of fixed size.
/// Finally, the prefix of this large const array is returned as a &'static [u8] slice cast to a &'static str.
///
/// We need to use a large const array *of fixed size* because:
/// - The array type in the implementation cannot depend on the generic const type parameter. (blocking feature: `generic_const_exprs`)
/// - Memory allocation is also of course not possible in const context.
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
