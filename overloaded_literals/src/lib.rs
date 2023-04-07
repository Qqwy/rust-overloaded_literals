extern crate self as overloaded_literals;
pub mod type_str;
use generic_array::ArrayLength;

pub use overloaded_literals_macro::overloaded_literals;
use type_str::TypeStr;

pub trait FromLiteralStr<TStr: TypeStr> {
    const VALID_LITERAL: &'static str;
    fn into_self() -> Self;
}

/// Build your datatype from an unsigned integer literal.
///
/// The [macro@overloaded_literals] macro turns unsigned integer literals like
/// ```rust
/// 1234
/// ```
/// into calls to
///
/// ```rust
/// FromLiteralUnsigned::<1234>::VALID_LITERAL::into_self()
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

/// Build your datatype from a signed integer literal.
///
/// The [macro@overloaded_literals] macro turns signed integer literals like
/// ```rust
/// -1234
/// ```
/// into calls to
///
/// ```rust
/// FromLiteralSigned::<-1234>::VALID_LITERAL::into_self()
/// ```
///
/// The first part (`VALID_LITERAL`) runs at compile-time, allowing you to perform input checks,
/// where invalid input results in a compile error.
///
/// The second part (`into_self()`) runs at runtime, and is where you create your actual value,
/// knowing that the input is guaranteed to be valid.
///
/// ```txt
/// FromLiteralSigned::<-1234>::VALID_LITERAL.into_self()
/// ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
///               compile time                 ^^^^^^^^^^^
///                                              runtime
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

impl<const LIT: u128> FromLiteralUnsigned<LIT> for u8 {
    const VALID_LITERAL: u128 = {
        let min = u8::MIN as u128;
        let max = i8::MAX as u128;
        if LIT < min || LIT > max {
            panic!("Out of range integer literal")
        } else {
            LIT
        }
    }; // u8_from_integer_literal::<LIT>();
    fn into_self() -> Self {
        <Self as FromLiteralUnsigned<LIT>>::VALID_LITERAL as u8
    }
}

impl<const LIT: u128> FromLiteralUnsigned<LIT> for i8 {
    const VALID_LITERAL: u128 = {
        let max = i8::MAX as u128;
        if LIT > max {
            panic!("Out of range integer literal")
        } else {
            LIT
        }
    };
    fn into_self() -> Self {
        <Self as FromLiteralUnsigned<LIT>>::VALID_LITERAL as i8
    }
}

impl<const LIT: i128> FromLiteralSigned<LIT> for i8 {
    const VALID_LITERAL: i128 = {
        let min = i8::MIN as i128;
        let max = i8::MAX as i128;
        if LIT < min || LIT > max {
            panic!("Out of range integer literal")
        } else {
            LIT
        }
    };
    fn into_self() -> Self {
        <Self as FromLiteralSigned<LIT>>::VALID_LITERAL as i8
    }
}

// Base definition
impl<'a, Str: TypeStr> FromLiteralStr<Str> for &'a str {
    const VALID_LITERAL: &'static str = Str::STR;
    fn into_self() -> Self {
        <Self as FromLiteralStr<Str>>::VALID_LITERAL
    }
}

// Build owned strings directly from string literals
impl<Str: TypeStr> FromLiteralStr<Str> for String {
    const VALID_LITERAL: &'static str = Str::STR;
    fn into_self() -> Self {
        <Self as FromLiteralStr<Str>>::VALID_LITERAL.to_string()
    }
}

// Simple example:
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Greeting {
    Hello,
    Goodbye,
}
impl<Str: TypeStr> FromLiteralStr<Str> for Greeting
where
    tlist::Len<Str>: ArrayLength<u8>,
{
    const VALID_LITERAL: &'static str = {
        let val = Str::STR;
        if const_str_eq(val, "hello") || const_str_eq(val, "goodbye") {
            val
        } else {
            panic!("Invalid Greeting literal");
        }
    };

    fn into_self() -> Self {
        let string = <Self as FromLiteralStr<Str>>::VALID_LITERAL;
        match string {
            "hello" => Greeting::Hello,
            "goodbye" => Greeting::Goodbye,
            _ => unreachable!(),
        }
    }
}

const fn const_str_eq(lhs: &str, rhs: &str) -> bool {
    if lhs.len() != rhs.len() {
        return false;
    }
    let len = lhs.len();
    let lhs_bytes = lhs.as_bytes();
    let rhs_bytes = rhs.as_bytes();
    let mut index = 0;
    while index < len {
        if lhs_bytes[index] != rhs_bytes[index] {
            return false;
        }
        index += 1;
    }
    true
}

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
        let y: Greeting = FromLiteralStr::<
            TList![Byte<104>, Byte<101>, Byte<108>, Byte<108>, Byte<111>],
        >::into_self();
        println!("greeting: {y:?}");
    }
}

// pub fn compile_time_error_on_invalid_inputs() {
//     let y: u8 = FromLiteralSigned::<1024>::into_self();
//     assert_eq!(y, 10);
// }

#[overloaded_literals]
pub fn example() -> i8 {
    let x = -1000;
    let _y: u8 = 123;
    x
}

#[overloaded_literals]
pub fn str_example() -> Greeting {
    let x: Greeting = "hello";
    // println!("{:?}", x);
    x
}

pub fn main() {
    let x = example();
    println!("x is: {x:?}");
}
