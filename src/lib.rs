//! Hotkey plugin entry point.

#![forbid(unsafe_code)]

#[cfg(target_arch = "wasm32")]
mod guest;

#[cfg(any(target_arch = "wasm32", test))]
#[allow(dead_code)]
mod actions;
