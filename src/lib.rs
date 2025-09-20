#![doc = include_str!("../README.md")]
#![allow(clippy::needless_doctest_main)]

// If neither `atomic` nor `encrypted` (which implies atomic) is enabled, fail fast.
#[cfg(all(not(feature = "atomic"), not(feature = "encrypted"),))]
compile_error!(
    "light-magic requires the `atomic` feature (or `encrypted`, which enables `atomic`). \
     Enable with: `features = [\"atomic\"]` or `--features atomic`."
);

#[cfg(feature = "atomic")]
pub use paste;
#[cfg(feature = "atomic")]
pub use serde;

#[cfg(feature = "atomic")]
pub mod atomic;
#[cfg(feature = "atomic")]
pub mod macros;
#[cfg(feature = "atomic")]
pub mod table;

#[cfg(feature = "encrypted")]
pub mod encrypted;
