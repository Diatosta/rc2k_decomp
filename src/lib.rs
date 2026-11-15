// `unsafe` only where a file says it may, each at the top of the file: the platform's edge
// and the few primitives the game logic stands on. `tools/count_unsafe.py` counts them.
#![cfg_attr(not(test), deny(unsafe_code))]
// Every `unsafe` block and impl says why it is sound (`// SAFETY:`), every `unsafe fn` and
// trait what its callers promise (`# Safety`), the tests' own too.
#![deny(clippy::undocumented_unsafe_blocks, clippy::missing_safety_doc)]

mod arch;
mod audio;
mod cells;
mod check_hash;
mod d3d;
mod debug;
mod drive;
mod effects;
mod errors;
mod fatal;
mod ffi;
mod files;
mod film;
mod framework;
mod gfx;
#[cfg(all(target_os = "linux", target_arch = "aarch64"))]
mod handheld;
mod hud;
mod input;
mod install;
mod list_handle;
mod logfile;
mod math;
mod memory;
mod menu;
mod net;
mod os;
mod owners;
mod part_cell;
mod physics;
mod place;
mod platform;
mod platform_3d;
mod platform_events;
mod platform_input;
mod platform_sound;
mod platform_timing;
mod pool;
mod race;
mod rasteriser;
#[cfg(target_arch = "aarch64")]
mod rasteriser_neon;
mod records;
#[cfg(feature = "remote")]
mod remote;
mod render;
mod saves;
mod scene;
mod screens;
mod service;
mod session;
mod settings;
mod shared_memory;
mod sound;
pub mod standalone;
mod strings;
mod sync;
mod system;
#[cfg(test)]
mod test_support;
mod text;
mod threading;
#[cfg(target_os = "emscripten")]
pub mod web;
mod workers;
