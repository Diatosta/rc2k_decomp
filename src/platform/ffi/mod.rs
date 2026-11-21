//! The outside libraries' `unsafe`, behind thin safe wrappers: one module per library. Outside
//! these and the memory primitives no file allows `unsafe`.

#![deny(clippy::undocumented_unsafe_blocks)]

#[cfg(target_os = "android")]
pub(crate) mod android;
#[cfg(target_os = "android")]
pub(crate) mod android_java;
pub(crate) mod arch;
#[cfg(target_os = "ios")]
pub(crate) mod dns_sd;
pub(crate) mod gl;
#[cfg(debug_ui)]
pub(crate) mod imgui;
#[cfg(handheld)]
pub(crate) mod linux_handheld;
#[cfg(rc2k_pgo)]
mod pgo;
#[cfg(phone)]
mod phone_vibrator;
pub(crate) mod photo_library;
pub(crate) mod process;
pub(crate) mod sdl;
pub(crate) mod sdl_camera;
pub(crate) mod sdl_clipboard;
#[cfg(not(handheld))]
pub(crate) mod sdl_gpu;
pub(crate) mod sdl_input;
#[cfg(unix)]
pub(crate) mod unix;
#[cfg(phone)]
mod vibrate;
#[cfg(web)]
pub mod web;
#[cfg(windows)]
pub(crate) mod win32;
