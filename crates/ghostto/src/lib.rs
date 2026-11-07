//! ghostto — one native binary, one localhost port.
//!
//! M0: facade crate. Re-exports the wire protocol and the version
//! constant. Engine crates (`ghostto_cpu_arm64`, `ghostto_mm`,
//! `ghostto_display`, ...) are added in M1+.

pub use ghostto_proto as proto;

/// Crate version, used in the browser handshake.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
