Overloaded Literals to construct your datatypes without boilerplate and with compile-time validation.

# Features
- Compile-time validation of literals
  - Supports `bool`, signed and unsigned integers and `&'static str`.
- Construct your types without ceremony or boilerplate.
- 100% `no_std` compatible.
- Runs on stable rust. MSRV: 1.65.0

Ships with implementations for `std`'s various [NonZero and Wrapping](https://doc.rust-lang.org/stable/std/num/index.html) structs

# Usage
Add the [overloaded_literals](https://docs.rs/overloaded_literals/latest/overloaded_literals/macro.overloaded_literals.html) as attribute to a function.
This will rewrite any literals to calls to a trait with the literal as generic const parameter.
Because a trait is used, construction of any desired target type which implements the type happens automatically:

```rust
use std::num::NonZeroI8;
use overloaded_literals::overloaded_literals;

#[overloaded_literals]
fn example() {
    let three: NonZeroI8 = 3;
    let result = three.saturating_mul(2); // <- This '2' also turns into a `NonZero` automatically because of the signature of `saturating_mul`.
    let six = 6; // <- And this '6' as well
    assert_eq!(result, six);
}
example()
```

Trait implementations can perform compile-time validation  (using 'const evaluation') on the passed literal.
This means that invalid literals are rejected at compile-time with a descriptive error message:

```compile_fail
use std::num::NonZeroI8;
use overloaded_literals::overloaded_literals;

#[overloaded_literals]
fn mistake() -> NonZeroI8 {
    let oops: NonZeroI8 = 0; // <- compile error 'NonZero integer literal was 0'.
    oops.saturating_mul(2)
}
mistake();
```

# Implementing the traits
As an example, here are the trait implementations for a type `EvenI32` which ensures that the value it stores is even, similarly to how [NonZeroI32](https://doc.rust-lang.org/stable/std/num/struct.NonZeroI32.html) ensures that the contained value is non-zero.

```rust
use overloaded_literals::{overloaded_literals, FromLiteralUnsigned, FromLiteralSigned};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct EvenI32(i32);

impl EvenI32 {
    fn new(val: i32) -> Option<Self> {
        if val % 2 != 0 {
            None
        } else {
            Some(EvenI32(val))
        }
    }
}

// Called for 0 and positive literals:
impl<const LIT: u128> FromLiteralUnsigned<LIT> for EvenI32 {
    const VALID_LITERAL: u128 = {
        if LIT % 2 != 0 {
            panic!("Odd EvenI32 integer literal")
        } else {
            LIT
        }
    };
    fn into_self() -> Self {
        let raw = <Self as FromLiteralUnsigned<LIT>>::VALID_LITERAL as i32;
        EvenI32(raw)
    }
}

// Called for negative literals:
impl<const LIT: i128> FromLiteralSigned<LIT> for EvenI32 {
    const VALID_LITERAL: i128 = {
        if LIT % 2 != 0 {
            panic!("Odd EvenI32 integer literal")
        } else {
            LIT
        }
    };
    fn into_self() -> Self {
        let raw = <Self as FromLiteralSigned<LIT>>::VALID_LITERAL as i32;
        EvenI32(raw)
    }
}


#[overloaded_literals]
fn example() {
    let x: EvenI32 = 100;
    // let y: EvenI32 = 7; // <- This would cause a compile error :-)
}
example()
```

Another full example, on how to accept a `str` literal for your datatype, can be found in the documentation of  [FromLiteralStr](https://docs.rs/overloaded_literals/latest/overloaded_literals/trait.FromLiteralStr.html).

# Missing features
The following features are currently missing and would be straightforward additions to later versions of the library:
- Support for `char` literals
- Support for float literals (_Requires some extra work since floats are not yet supported in generic const contexts._)
- Support for raw byte str literals (_Requires a similar abstraction as [TypeStr](https://docs.rs/overloaded_literals/latest/overloaded_literals/type_str/trait.TypeStr.html)._)
- Implementations of `FromLiteralStr` for `CStr` and other `str`-like types.
