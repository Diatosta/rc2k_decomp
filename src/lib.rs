// `unsafe` only where a file says it may, each at the top of the file: the platform's edge
// and the few primitives the game logic stands on. `tools/count_unsafe.py` counts them.
#![cfg_attr(not(test), deny(unsafe_code))]
// Every `unsafe` block and impl says why it is sound (`// SAFETY:`), every `unsafe fn` and
// trait what its callers promise (`# Safety`), the tests' own too.
#![deny(clippy::undocumented_unsafe_blocks, clippy::missing_safety_doc)]

mod audio;
mod d3d;
mod debug;
mod drive;
mod effects;
mod error_dialog;
mod errors;
mod fatal;
mod files;
mod first_run;
mod gfx;
mod hud;
mod input;
mod install;
mod logfile;
mod math;
mod memory;
mod menu;
mod net;
mod owners;
mod physics;
mod platform;
mod race;
mod records;
#[cfg(feature = "remote")]
mod remote;
mod render;
mod save_code;
mod saves;
mod scene;
mod screens;
mod service;
mod settings;
mod single_copy;
mod sound;
pub mod standalone;
mod strings;
mod system;
#[cfg(test)]
mod test_support;
mod text;
mod threading;
mod video;
#[cfg(target_os = "android")]
pub use platform::android;
#[cfg(web)]
pub use platform::web;
