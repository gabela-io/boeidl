//! Includes the generated module for Modelo 130.
//!
//! The generated file is wrapped in a private inner module so the `allow`
//! attributes don't leak out to the rest of the crate, and then re-exported.

#[allow(unused, clippy::all, unused_parens, dead_code, non_snake_case)]
mod inner {
    include!(concat!(env!("OUT_DIR"), "/mod130.rs"));
}

pub use inner::*;
