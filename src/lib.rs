// Nearly every function here is `unsafe` for one and the same reason: it reads and writes the
// game's memory at fixed addresses, so it is only sound inside the running game, on data the
// game has loaded, under the conditions its header comment gives. A "# Safety" section on each
// of some 130 functions would repeat that sentence; the particular conditions are in each
// function's own doc comment and in the `Err(BadInput)` cases it checks.
#![allow(clippy::missing_safety_doc)]

mod audio;
mod bridge;
mod calls;
mod d3d;
mod drive;
mod errors;
mod fatal;
mod files;
mod film;
mod framework;
mod gfx;
mod hud;
mod input;
mod install;
mod logfile;
mod marks;
mod math;
mod memory;
mod menu;
mod moved;
mod net;
mod physics;
mod platform;
mod platform_3d;
mod platform_input;
mod platform_sound;
mod race;
mod rasteriser;
mod records;
mod registry;
mod relocations;
mod rest;
mod routes;
mod saves;
mod scene;
mod screens;
mod service;
mod session;
mod settings;
mod sound;
pub mod standalone;
mod store;
mod strings;
mod system;
mod text;
mod threading;

use crate::framework::BASE;
use crate::registry::ALL;
use std::sync::atomic::Ordering;

/// The program's hooks counted as in place at `base` (the image holds no code to patch:
/// `standalone.rs` sends each hooked address to the hook's thunk).
pub fn mark_all_installed(base: usize) -> Vec<bool> {
    BASE.store(base, Ordering::Relaxed);
    vec![true; ALL.len()]
}
