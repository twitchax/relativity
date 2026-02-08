//! Shared test utilities for E2E tests.
//!
//! Sub-modules:
//! - [`gameplay`]: Helpers for headless gameplay tests (`MinimalPlugins` + `GamePlugin`).
//! - [`headless`]: Helpers for headless render tests (`DefaultPlugins` − `WinitPlugin`).
//! - [`screenshot`]: Screenshot comparison / visual regression utilities.

#![allow(clippy::unwrap_used)]
#![allow(unused_imports)]

pub mod gameplay;
pub mod headless;
pub mod screenshot;

// Re-export gameplay helpers for backward compatibility with existing tests.
pub use gameplay::*;
