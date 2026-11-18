//! The outside libraries' `unsafe`, behind thin safe wrappers: one module per library. Outside
//! these and the memory primitives no file allows `unsafe`.

#![deny(clippy::undocumented_unsafe_blocks)]

#[cfg(target_os = "android")]
pub mod android;
pub mod gl;
#[cfg(debug_ui)]
pub mod imgui;
#[cfg(all(target_os = "linux", target_arch = "aarch64"))]
pub mod linux_handheld;
#[cfg(rc2k_pgo)]
pub mod pgo;
pub mod sdl;
pub mod sdl_camera;
pub mod sdl_clipboard;
#[cfg(not(all(target_os = "linux", target_arch = "aarch64")))]
pub mod sdl_gpu;
pub mod sdl_input;
#[cfg(unix)]
pub mod unix;
#[cfg(windows)]
pub mod win32;
