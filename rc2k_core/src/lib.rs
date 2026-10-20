//! The game's logic as plain Rust (owner, 2026-10-20: the hook framework removed module by
//! module; `notes/conversion.md`). No addresses, no platform, no hooks: the state is in
//! ordinary Rust structures, so it builds and tests on Linux and Windows. While the hooked
//! DLL checks the conversion, the hook crate's adapters (`src/bridge.rs`) load these
//! structures from the game's memory and store them back.

pub mod audio;
pub mod files;
pub mod gfx;
pub mod input;
pub mod math;
pub mod screens;
pub mod sound;
pub mod strings;
pub mod text;

/// An input the core does not take: one the original goes wrong on (a read past a table, a
/// division that traps) or whose handling was not worked out; the reason in words.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Invalid(pub &'static str);
