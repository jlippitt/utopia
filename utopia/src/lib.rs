#![allow(
    clippy::collapsible_else_if,
    clippy::comparison_chain,
    clippy::identity_op,
    clippy::manual_range_contains,
    clippy::match_single_binding,
    clippy::neg_multiply,
    clippy::single_match
)]

pub use system::*;

pub mod core;
mod system;
mod util;
