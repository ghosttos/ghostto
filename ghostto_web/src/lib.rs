//! WebAssembly bindings for ghostto.
//! Placeholder for v0.0.2 — real engine bindings land here.

use wasm_bindgen::prelude::*;

/// Returns the package version, exposed to JavaScript.
#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Returns a greeting so the JS API has something callable.
#[wasm_bindgen]
pub fn hello() -> String {
    "ghostto".to_string()
}
