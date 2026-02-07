// Deny unwrap_used in production code to ensure proper error handling.
// Test code is allowed to use unwrap via #[cfg(test)] and module-level allows.
#![deny(unused)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::correctness)]
#![deny(clippy::complexity)]
#![deny(clippy::pedantic)]

pub mod game;
pub mod menu;
pub mod shared;
