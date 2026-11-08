//! The race's effects (weather, spray, windscreen drops, lens flare; masks, splines, noise):
//! the converted logic of the hook crate's `effects` (called "the marks" until 2026-10-29, the
//! game's "init mark").

pub mod clock;
pub mod drift;
pub mod expiry;
pub mod fades;
pub mod items;
pub mod kind_values;
pub mod lines;
pub mod masks;
pub mod noise;
pub mod particles;
pub mod points;
pub mod pong;
pub mod race;
pub mod rings;
pub mod scales;
pub mod shapes;
pub mod slots;
pub mod splines;
pub mod steps;
pub mod turns;
pub mod view;
pub mod waves;
