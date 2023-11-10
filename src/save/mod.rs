#[cfg(not(target_arch = "wasm32"))]
mod save_os;
#[cfg(target_arch = "wasm32")]
mod save_web;

// Re-export the platform-specific implementation
#[cfg(target_arch = "wasm32")]
pub use crate::save::save_web::*;

#[cfg(not(target_arch = "wasm32"))]
pub use crate::save::save_os::*;
