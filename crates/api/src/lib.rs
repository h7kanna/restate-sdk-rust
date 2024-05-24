//! Restate Rust SDK API

pub use restate_sdk::{context::RestateContext as Context, endpoint};
#[cfg(feature = "derive")] pub use restate_sdk_derive::main;
