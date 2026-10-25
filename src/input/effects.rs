//! The controllers' force feedback, the game's side (read 2026-10-12, ported 2026-10-19):
//! what the race asks of the six effects of each device record (+0x3C, 0xA4 bytes each).
//! A request is a dword at the effect's +0x2C (`input::set_effect`: what to do in the low
//! byte, 2 start, 3 set, 4 stop; two signed 12-bit numbers above it), which the input
//! thread hands to DirectInput (`input::device_effects`, the platform's side). Nothing
//! here calls DirectInput.
//!
//! The calls by place (the callee's IDA address): in the game the Rust versions, in the
//! tests the noting stand-ins of `sound::turn_parts`.

/// The record's +0x28: 1 while it has effects.
pub const HAS_EFFECTS: u32 = 0x28;
pub const EFFECTS: u32 = 0x3C;
pub const EFFECT_LEN: u32 = 0xA4;
pub const REQUEST: u32 = 0x2C;
