// `unsafe` only where a file says it may, each at the top of the file: the platform's edge
// and the few primitives the game logic stands on. `tools/count_unsafe.py` counts them.
#![cfg_attr(not(test), deny(unsafe_code))]

mod address_map;
mod arch;
mod audio;
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
mod logfile;
mod math;
mod memory;
mod menu;
mod moved;
mod net;
mod os;
mod physics;
mod place;
mod platform;
mod platform_3d;
mod platform_events;
mod platform_input;
mod platform_sound;
mod platform_timing;
mod pointers;
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
mod store;
mod strings;
mod sync;
mod system;
mod text;
mod threading;
#[cfg(target_os = "emscripten")]
pub mod web;
mod workers;
mod write_watch;
