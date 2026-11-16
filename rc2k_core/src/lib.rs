//! The game's logic as plain Rust. No addresses, no platform: the state is in ordinary Rust
//! structures, so it builds and tests on Linux and Windows.

#![cfg_attr(not(test), forbid(unsafe_code))]

pub mod audio;
pub mod cli;
pub mod d3d;
pub mod drive;
pub mod effects;
pub mod fatal;
pub mod files;
pub mod film;
pub mod gfx;
pub mod hud;
pub mod input;
pub mod install;
pub mod math;
pub mod memory;
pub mod menu;
pub mod net;
pub mod options;
pub mod physics;
pub mod race;
pub mod saves;
pub mod scene;
pub mod screens;
pub mod service;
pub mod sound;
pub mod strings;
pub mod system;
pub mod threading;

/// An input the core does not take (a read past a table, a division that traps) or whose
/// handling is not worked out; the reason in words.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Invalid(pub &'static str);
