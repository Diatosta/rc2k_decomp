//! The outside libraries' `unsafe`, behind thin safe wrappers: one module per library. Outside
//! these and the memory primitives no file allows `unsafe`.

#![deny(clippy::undocumented_unsafe_blocks)]

#[cfg(target_os = "android")]
pub mod android;
#[cfg(target_os = "android")]
pub mod android_java;
#[cfg(target_os = "ios")]
pub mod dns_sd;
pub mod gl;
#[cfg(debug_ui)]
pub mod imgui;
#[cfg(handheld)]
pub mod linux_handheld;
#[cfg(rc2k_pgo)]
pub mod pgo;
#[cfg(phone)]
pub mod phone_vibrator;
pub mod photo_library;
pub mod sdl;
pub mod sdl_camera;
pub mod sdl_clipboard;
#[cfg(not(handheld))]
pub mod sdl_gpu;
pub mod sdl_input;
#[cfg(unix)]
pub mod unix;
#[cfg(windows)]
pub mod win32;
