#![doc = include_str!("../../README.md")]
#![no_std]

#[cfg(test)]
extern crate std;
#[cfg(test)]
use std::println;

extern crate self as overloaded_literals;
pub mod type_str;
use core::num::{NonZeroU8, NonZeroU16, NonZeroU32, NonZeroU64, NonZeroU128, NonZeroUsize};
use core::num::{NonZeroI8, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI128, NonZeroIsize};
use core::num::Wrapping;

pub use type_str::TypeStr;

/// Attribute macro to overload literals in the function it is used on.
///
/// # Usage
///
/// ```ignore
/// use overloaded_literals::overloaded_literals;
/// #[overloaded_literals]
/// pub fn example() {
///    let x: NonZeroU8 = 10;
///    println!("x is: {:?}", &x);
///    let y: NonZeroIsize = -42;
///    println!("y is: {:?}", &y);
///    let z: Greeting = "hello";
///    println!("z is: {:?}", &z);
/// }
/// ```
///
/// - Any *unsigned* integer literal like `1337` is rewritten to [`FromLiteralUnsigned::<1337>::into_self()`](FromLiteralUnsigned)
/// - Any *unsigned* integer literal like `-4200` is rewritten to [`FromLiteralSigned::<-4200>::into_self()`](FromLiteralSigned)
/// - Any `str` literal like `"hello"` is rewritten to [`FromLiteralStr::<"hello">::into_self()`](FromLiteralStr)
pub use overloaded_literals_macro::overloaded_literals;


mod sealed {
    pub trait Sealed {}
    impl<const VAL: u8> Sealed for crate::type_str::Byte<VAL> {}
    impl Sealed for tlist::TNil {}
    impl<H, T: tlist::TList> Sealed for tlist::TCons<H, T> {}
}

/// Build your datatype from a `&'static str` literal.
///
/// The [macro@overloaded_literals] macro turns signed integer literals like
/// ```compile_only
/// "hello"
/// ```
/// into calls to
///
/// ```compile_only
/// FromLiteralStr::<"hello">::VALID_LITERAL::into_self()
/// ```
/// _NOTE: Because stable rust does not currently support generic const static str literals,
/// the real desugaring is slightly more involved, see [TypeStr] if you are curious._
///
/// The first part (`VALID_LITERAL`) runs at compile-time, allowing you to perform input checks,
/// where invalid input results in a compile error.
///
/// The second part (`into_self()`) runs at runtime, and is where you create your actual value,
/// knowing that the input is guaranteed to be valid.
///
/// ```txt
/// FromLiteralStr::<-1234>::VALID_LITERAL.into_self()
/// ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
///               compile time             ^^^^^^^^^^^
///                                          runtime
/// ```
///
/// # Example
/// As an example, consider a simple enum, whose valid values are `"hello"` and `"goodbye".`
/// Since the amount of builtin const operations that are allowed on `str` is currently limited on stable rust,
/// we use the [const_str](https://crates.io/crates/const-str) crate for these.
/// ```rust
/// use overloaded_literals::{overloaded_literals, FromLiteralStr, TypeStr};
///
/// #[derive(Debug, Clone, PartialEq, Eq)]
/// pub enum Greeting {
///     Hello,
///     Goodbye,
/// }
///
/// impl<Str: TypeStr> FromLiteralStr<Str> for Greeting
/// {
///     const VALID_LITERAL: &'static str = {
///         let val = Str::STR;
///         if const_str::equal!(val, "hello") || const_str::equal!(val, "goodbye") {
///             val
///         } else {
///             panic!("Invalid Greeting literal; only `hello` and `goodbye` are allowed.");
///         }
///     };
///
///     fn into_self() -> Self {
///         let string = <Self as FromLiteralStr<Str>>::VALID_LITERAL;
///         match string {
///             "hello" => Greeting::Hello,
///             "goodbye" => Greeting::Goodbye,
///             _ => unreachable!(),
///         }
///     }
/// }
/// // Usage:
/// #[overloaded_literals]
/// fn example() {
///    let val: Greeting = "hello";
///    let other_val: Greeting = "goodbye";
///    // let boom: Greeting = "hehehehehe"; // <- This would cause a compile error :-)
/// }
/// example()
/// ```
pub trait FromLiteralStr<TStr: TypeStr> {
    /// The definition of `VALID_LITERAL` is evaluated at compile-time.
    ///
    /// Inside this definition you have access to `TStr::STR` which returns a `&'static str`.
    ///
    /// An implementation of `VALID_LITERAL` should perform input checking:
    /// - If the input is valid, return `TStr::STR` unchanged.
    /// - If the input is invalid, [panic](core::panic!).
    ///   Because this is evaluated at compile-time, this results in a compile error.
    ///
    /// Since not many operations on `&'static str` are stably allowed in const contexts yet,
    /// you might want to use crates like [const-str](https://crates.io/crates/const-str).
    const VALID_LITERAL: &'static str;

    /// Turns a [VALID_LITERAL](FromLiteralUnsigned::VALID_LITERAL) into the actual runtime value.
    ///
    /// This part runs at runtime.
    ///
    /// You have access to [VALID_LITERAL](FromLiteralStr::VALID_LITERAL) (using the syntax `let val = <Self as FromLiteralStr<TStr>>::VALID_LITERAL;`),
    /// and should turn it into your desired value.
    ///
    /// If you want, you can use an unsafe 'unchecked' constructor, if one exists, since you have done any validation already.
    /// (But even if using a normal constructor, in all likelyhood the compiler is smart enough to remove the duplicate checks since the input is a literal value.)
    fn into_self() -> Self;
}

// Base definition
impl<'a, Str: TypeStr> FromLiteralStr<Str> for &'a str {
    const VALID_LITERAL: &'static str = Str::STR;
    fn into_self() -> Self {
        <Self as FromLiteralStr<Str>>::VALID_LITERAL
    }
}

// Build owned strings directly from string literals
// impl<Str: TypeStr> FromLiteralStr<Str> for String {
//     const VALID_LITERAL: &'static str = Str::STR;
//     fn into_self() -> Self {
//         <Self as FromLiteralStr<Str>>::VALID_LITERAL.to_string()
//     }
// }


/// Build your datatype from an unsigned integer literal (0, 1, 2, 3, ...).
///
/// The [macro@overloaded_literals] macro turns unsigned integer literals like
/// ```compile_only
/// 1234
/// ```
/// into calls to
///
/// ```compile_only
/// FromLiteralStr::<1234>::VALID_LITERAL::into_self()
/// ```
///
/// The first part (`VALID_LITERAL`) runs at compile-time, allowing you to perform input checks,
/// where invalid input results in a compile error.
///
/// The second part (`into_self()`) runs at runtime, and is where you create your actual value,
/// knowing that the input is guaranteed to be valid.
///
/// ```txt
/// FromLiteralUnsigned::<1234>::VALID_LITERAL.into_self()
/// ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
///               compile time                 ^^^^^^^^^^^
///                                              runtime
/// ```
///
pub trait FromLiteralUnsigned<const LIT: u128> {
    /// The definition of `VALID_LITERAL` is evaluated at compile-time.
    ///
    /// Inside this definition you have access to `LIT`.
    ///
    /// An implementation of `VALID_LITERAL` should perform input checking:
    /// - If the input is valid, return `LIT` unchanged.
    /// - If the input is invalid, [panic](core::panic!).
    ///   Because this is evaluated at compile-time, this results in a compile error.
    const VALID_LITERAL: u128;

    /// Turns a [VALID_LITERAL](FromLiteralUnsigned::VALID_LITERAL) into the actual runtime value.
    ///
    /// This part runs at runtime.
    ///
    /// You have access to [VALID_LITERAL](FromLiteralUnsigned::VALID_LITERAL) (using the syntax `let val = <Self as FromLiteralUnsigned<LIT>>::VALID_LITERAL;`),
    /// and should turn it into your desired value.
    ///
    /// If you want, you can use an unsafe 'unchecked' constructor, if one exists, since you have done any validation already.
    /// (But even if using a normal constructor, in all likelyhood the compiler is smart enough to remove the duplicate checks since the input is a literal value.)
    fn into_self() -> Self;
}

/// Build your datatype from a signed integer literal (-1, -2, -3, ...).
///
/// The [macro@overloaded_literals] macro turns signed integer literals like
/// ```compile_only
/// -1234
/// ```
/// into calls to
///
/// ```compile_only
/// FromLiteralSigned::<-1234>::VALID_LITERAL::into_self()
/// ```
///
/// **NOTE: In the common case where the target type also accepts 0 or positive integer literals, it should also implement [FromLiteralUnsigned].**
///
/// The first part (`VALID_LITERAL`) runs at compile-time, allowing you to perform input checks,
/// where invalid input results in a compile error.
///
/// The second part (`into_self()`) runs at runtime, and is where you create your actual value,
/// knowing that the input is guaranteed to be valid.
///
/// ```txt
/// FromLiteralSigned::<-1234>::VALID_LITERAL.into_self()
/// ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
///               compile time                ^^^^^^^^^^^
///                                             runtime
/// ```
///
pub trait FromLiteralSigned<const LIT: i128> {
    /// The definition of `VALID_LITERAL` is evaluated at compile-time.
    ///
    /// Inside this definition you have access to `LIT`.
    ///
    /// An implementation of `VALID_LITERAL` should perform input checking:
    /// - If the input is valid, return `LIT` unchanged.
    /// - If the input is invalid, [panic](core::panic!).
    ///   Because this is evaluated at compile-time, this results in a compile error.
    const VALID_LITERAL: i128;

    /// Turns a [VALID_LITERAL](FromLiteralSigned::VALID_LITERAL) into the actual runtime value.
    ///
    /// This part runs at runtime.
    ///
    /// You have access to [VALID_LITERAL](FromLiteralSigned::VALID_LITERAL) (using the syntax `let val = <Self as FromLiteralSigned<LIT>>::VALID_LITERAL;`),
    /// and should turn it into your desired value.
    ///
    /// If you want, you can use an unsafe 'unchecked' constructor, if one exists, since you have done any validation already.
    /// (But even if using a normal constructor, in all likelyhood the compiler is smart enough to remove the duplicate checks since the input is a literal value.)
    fn into_self() -> Self;
}

macro_rules! unsigned_impl {
    ($type:ty) => {
        impl<const LIT: u128> FromLiteralUnsigned<LIT> for $type {
            const VALID_LITERAL: u128 = {
                let min = <$type>::MIN as u128;
                let max = <$type>::MAX as u128;
                if LIT < min || LIT > max {
                    panic!("Out of range integer literal")
                } else {
                    LIT
                }
            };
            fn into_self() -> Self {
                <Self as FromLiteralUnsigned<LIT>>::VALID_LITERAL as $type
            }
        }
    }
}


macro_rules! signed_impl {
    ($type:ty) => {
        impl<const LIT: i128> FromLiteralSigned<LIT> for $type {
            const VALID_LITERAL: i128 = {
                let min = <$type>::MIN as i128;
                let max = <$type>::MAX as i128;
                if LIT < min || LIT > max {
                    panic!("Out of range integer literal")
                } else {
                    LIT
                }
            };
            fn into_self() -> Self {
                <Self as FromLiteralSigned<LIT>>::VALID_LITERAL as $type
            }
        }
    }
}

unsigned_impl!(u8);
unsigned_impl!(u16);
unsigned_impl!(u32);
unsigned_impl!(u64);
unsigned_impl!(u128);
unsigned_impl!(usize);


unsigned_impl!(i8);
unsigned_impl!(i16);
unsigned_impl!(i32);
unsigned_impl!(i64);
unsigned_impl!(i128);
unsigned_impl!(isize);

signed_impl!(i8);
signed_impl!(i16);
signed_impl!(i32);
signed_impl!(i64);
signed_impl!(i128);
signed_impl!(isize);


macro_rules! nonzero_unsigned_impl {
    ($type:ty, $orig_type:ty) => {
        impl<const LIT: u128> FromLiteralUnsigned<LIT> for $type {
            const VALID_LITERAL: u128 = {
                let max = <$orig_type>::MAX as u128;
                if LIT == 0 {
                    panic!("NonZero integer literal was 0")
                }
                if LIT > max {
                    panic!("Out of range NonZero integer literal")
                } else {
                    LIT
                }
            };
            fn into_self() -> Self {
                let raw = <Self as FromLiteralUnsigned<LIT>>::VALID_LITERAL as $orig_type;
                // SAFETY: Bounds check happened at compile time
                unsafe { <$type>::new_unchecked(raw) }
            }
        }
    }
}

macro_rules! nonzero_signed_impl {
    ($type:ty, $orig_type:ty) => {
        impl<const LIT: i128> FromLiteralSigned<LIT> for $type {
            const VALID_LITERAL: i128 = {
                let min = <$orig_type>::MIN as i128;
                let max = <$orig_type>::MAX as i128;
                if LIT == 0 {
                    panic!("NonZero integer literal was 0")
                }
                if LIT < min || LIT > max {
                    panic!("Out of range NonZero integer literal")
                } else {
                    LIT
                }
            };
            fn into_self() -> Self {
                let raw = <Self as FromLiteralSigned<LIT>>::VALID_LITERAL as $orig_type;
                // SAFETY: Bounds check happened at compile time
                unsafe { <$type>::new_unchecked(raw) }
            }
        }
    }
}

nonzero_unsigned_impl!(NonZeroU8, u8);
nonzero_unsigned_impl!(NonZeroU16, u16);
nonzero_unsigned_impl!(NonZeroU32, u32);
nonzero_unsigned_impl!(NonZeroU64, u64);
nonzero_unsigned_impl!(NonZeroU128, u128);
nonzero_unsigned_impl!(NonZeroUsize, usize);

nonzero_unsigned_impl!(NonZeroI8, i8);
nonzero_unsigned_impl!(NonZeroI16, i16);
nonzero_unsigned_impl!(NonZeroI32, i32);
nonzero_unsigned_impl!(NonZeroI64, i64);
nonzero_unsigned_impl!(NonZeroI128, i128);
nonzero_unsigned_impl!(NonZeroIsize, isize);
nonzero_signed_impl!(NonZeroI8, i8);
nonzero_signed_impl!(NonZeroI16, i16);
nonzero_signed_impl!(NonZeroI32, i32);
nonzero_signed_impl!(NonZeroI64, i64);
nonzero_signed_impl!(NonZeroI128, i128);
nonzero_signed_impl!(NonZeroIsize, isize);

impl<T: FromLiteralUnsigned<LIT>, const LIT: u128> FromLiteralUnsigned<LIT> for Wrapping<T>
{
    const VALID_LITERAL: u128 = T::VALID_LITERAL;
    fn into_self() -> Self {
        Wrapping(T::into_self())
    }
}

impl<T: FromLiteralSigned<LIT>, const LIT: i128> FromLiteralSigned<LIT> for Wrapping<T>
{
    const VALID_LITERAL: i128 = T::VALID_LITERAL;
    fn into_self() -> Self {
        Wrapping(T::into_self())
    }
}

/// Build your datatype from a boolean literal (`false` or `true`).
///
/// The [macro@overloaded_literals] macro turns boolean literals like
/// ```compile_only
/// true
/// ```
/// into calls to
///
/// ```compile_only
/// FromLiteralBool::<true>::VALID_LITERAL::into_self()
/// ```
///
///
/// The first part (`VALID_LITERAL`) runs at compile-time, allowing you to perform input checks,
/// where invalid input results in a compile error.
///
/// The second part (`into_self()`) runs at runtime, and is where you create your actual value,
/// knowing that the input is guaranteed to be valid.
///
/// ```txt
/// FromLiteralBool::<true>::VALID_LITERAL.into_self()
/// ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
///               compile time             ^^^^^^^^^^^
///                                          runtime
/// ```
///
pub trait FromLiteralBool<const LIT: bool> {
    /// The definition of `VALID_LITERAL` is evaluated at compile-time.
    ///
    /// Inside this definition you have access to `LIT`.
    ///
    /// An implementation of `VALID_LITERAL` should perform input checking:
    /// - If the input is valid, return `LIT` unchanged.
    /// - If the input is invalid, [panic](core::panic!).
    ///   Because this is evaluated at compile-time, this results in a compile error.
    const VALID_LITERAL: bool;

    /// Turns a [VALID_LITERAL](FromLiteralBool::VALID_LITERAL) into the actual runtime value.
    ///
    /// This part runs at runtime.
    ///
    /// You have access to [VALID_LITERAL](FromLiteralBool::VALID_LITERAL) (using the syntax `let val = <Self as FromLiteralBool<LIT>>::VALID_LITERAL;`),
    /// and should turn it into your desired value.
    ///
    /// If you want, you can use an unsafe 'unchecked' constructor, if one exists, since you have done any validation already.
    /// (But even if using a normal constructor, in all likelyhood the compiler is smart enough to remove the duplicate checks since the input is a literal value.)
    fn into_self() -> Self;
}

impl<const LIT: bool> FromLiteralBool<LIT> for bool {
    const VALID_LITERAL: bool = LIT;
    fn into_self() -> Self {
        <Self as FromLiteralBool<LIT>>::VALID_LITERAL
    }
}

// pub trait FromLiteralFloat<const LIT: f64> {
//     /// The definition of `VALID_LITERAL` is evaluated at compile-time.
//     ///
//     /// Inside this definition you have access to `LIT`.
//     ///
//     /// An implementation of `VALID_LITERAL` should perform input checking:
//     /// - If the input is valid, return `LIT` unchanged.
//     /// - If the input is invalid, [panic](core::panic!).
//     ///   Because this is evaluated at compile-time, this results in a compile error.
//     const VALID_LITERAL: f64;

//     /// Turns a [VALID_LITERAL](FromLiteralSigned::VALID_LITERAL) into the actual runtime value.
//     ///
//     /// This part runs at runtime.
//     ///
//     /// You have access to [VALID_LITERAL](FromLiteralSigned::VALID_LITERAL) (using the syntax `let val = <Self as FromLiteralSigned<LIT>>::VALID_LITERAL;`),
//     /// and should turn it into your desired value.
//     ///
//     /// If you want, you can use an unsafe 'unchecked' constructor, if one exists, since you have done any validation already.
//     /// (But even if using a normal constructor, in all likelyhood the compiler is smart enough to remove the duplicate checks since the input is a literal value.)
//     fn into_self() -> Self;
// }



// unsigned_impl!(NonZeroU16);
// unsigned_impl!(NonZeroU32);
// unsigned_impl!(NonZeroU64);
// unsigned_impl!(NonZeroUsize);


// use core::ffi::CStr;
// impl<Str: TypeStr> FromLiteralStr<Str> for &'static CStr {
//     const VALID_LITERAL: &'static str = {
//         let val = Str::STR;
//         let val_start = &val.as_bytes()[0..10];
//         // let penultimate = val.len() - 1;
//         if const_str::contains!(val, '\0') {
//             panic!("Invalid CStr literal: Contains NUL bytes in the middle.")
//         }
//         // if val.as_bytes()[val.len() - 1] != 0 {
//         if !const_str::ends_with!(val, '\0') {
//             panic!("Invalid CStr literal: Missing terminating NUL byte.")
//         }
//         val
//     };
//     fn into_self() -> Self {
//         let lit = <Self as FromLiteralStr<Str>>::VALID_LITERAL;
//         unsafe { CStr::from_bytes_with_nul_unchecked(lit.as_bytes()) }
//     }
// }

// const fn const_str_eq(lhs: &str, rhs: &str) -> bool {
//     if lhs.len() != rhs.len() {
//         return false;
//     }
//     let len = lhs.len();
//     let lhs_bytes = lhs.as_bytes();
//     let rhs_bytes = rhs.as_bytes();
//     let mut index = 0;
//     while index < len {
//         if lhs_bytes[index] != rhs_bytes[index] {
//             return false;
//         }
//         index += 1;
//     }
//     true
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        // let result = add(2, 2);
        let x: i8 = FromLiteralSigned::<2>::into_self();
        assert_eq!(x, 2);
        let x: i8 = FromLiteralSigned::<-20>::into_self();
        assert_eq!(x, -20);
        // let x: i8 = FromLiteralSigned::<-200>::into_self();
    }

    // #[test]
    // fn compile_time_error_on_invalid_inputs() {
    //     let y: u8 = FromLiteralSigned::<1024>::into_self();
    //     assert_eq!(y, 10);
    // }

    // #[test]
    // pub fn string_example() {
    //     let val = <TList![Byte<65>, Byte<66>, Byte<67>] as TypeStr>::STR;
    //     println!("val: {:?}", val);
    // }
    #[test]
    fn literal_str() {
        use tlist::TList;
        use type_str::Byte;
        // [103, 114, 101, 101, 116, 105, 110, 103, 0]
        // let y: Greeting = FromLiteralStr::<TList![Char<103>, Char<114>, Char<101>, Char<101>, Char<116>, Char<105>, Char<110>, Char<103>]>::into_self();
        let y: &'static str = FromLiteralStr::<
            TList![Byte<104>, Byte<101>, Byte<108>, Byte<108>, Byte<111>],
        >::into_self();
        println!("greeting: {y:?}");
    }
}


// #[overloaded_literals]
// fn bar() {
//     let res: (f32, f32, f32) = (1.0, -42.0, 10e3.0);
//     res
// }
