//! Code generation backends.
//!
//! For V1 there is only one backend (`rust`), which emits a Rust module
//! containing the model struct plus marshal/unmarshal/validate/compute_derived.

pub mod rust;
