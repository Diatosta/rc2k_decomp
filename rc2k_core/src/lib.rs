//! The game's logic as plain Rust (owner, 2026-10-20: the hook framework removed module by
//! module; `notes/conversion.md`). No addresses, no platform, no hooks: the state is in
//! ordinary Rust structures, so it builds and tests on Linux and Windows. While the hooked
//! DLL checks the conversion, the hook crate's adapters (`src/bridge.rs`) load these
//! structures from the game's memory and store them back.

pub mod best_scores;
pub mod blit;
pub mod buttons;
pub mod bytes;
pub mod captions;
pub mod case;
pub mod devices;
pub mod extended;
pub mod feedback;
pub mod format;
pub mod geometry;
pub mod glyphs;
pub mod input;
pub mod keyboard;
pub mod language;
pub mod lexicon;
pub mod log;
pub mod mixer;
pub mod packed;
pub mod paths;
pub mod pcx;
pub mod picture;
pub mod pixels;
pub mod png;
pub mod random;
pub mod raster;
pub mod records;
pub mod samples;
pub mod screen;
pub mod settings;
pub mod setup_markers;
pub mod spot_voices;
pub mod stage_panel;
pub mod strings;
pub mod text;
pub mod units;
pub mod vector;

/// An input the core does not take: one the original goes wrong on (a read past a table, a
/// division that traps) or whose handling was not worked out; the reason in words.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Invalid(pub &'static str);
