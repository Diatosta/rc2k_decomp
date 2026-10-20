//! The game's logic as plain Rust (owner, 2026-10-20: the hook framework removed module by
//! module; `notes/conversion.md`). No addresses, no platform, no hooks: the state is in
//! ordinary Rust structures, so it builds and tests on Linux and Windows. While the hooked
//! DLL checks the conversion, the hook crate's adapters (`src/bridge.rs`) load these
//! structures from the game's memory and store them back.

pub mod case;
pub mod format;
pub mod random;
pub mod units;
