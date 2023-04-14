# HEAD

Doc:
- Fixed a typo in the documentation (PR [#1](https://github.com/Qqwy/rust-overloaded_literals/pull/1), thank you, @vortexofdoom!)

# 0.8.2

Minor:
- Improve in-lib implementations and example implementations to use `assert!` instead of `panic!` for extra clarity.
- Add `#[inline]` decorations to all calls to `into_self()`. This is probably usually not necessary, but good style :-).

Fix:
- Make sure suffixes of int/float literals are respected (overloading is only attempted on literals without a suffix)

# 0.8.1

- Fix syntax highlighting in README on crates.io page

# 0.8.0

- `FromLiteralStr` support for `core::ffi::CStr`.

# 0.7.1

- Turn off unused (debug-only) feature flag from `syn` dependency, resulting in faster builds.

# 0.7.0

Minor:
- Bump `tlist` dependency version and as a result remove now no-longer-necessary `typenum` dependency, resulting in less dependencies.

# 0.6.0

Features:
- Float support
