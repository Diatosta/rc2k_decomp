//! `physics::fill_slot_on_bit` (0x462237): when bit cl of `g_car` +0x298 is set, it is
//! cleared and part 0x50 + 0x14 x `g_currentCar` + cl goes into a race slot
//! (`physics::fill_race_slot`).
//!
//! In:  cl the bit (ecx passed on)
//! Out: eax 1 << cl, or as `fill_race_slot` leaves them; no flags read
//!
//! Callers: `physics::drop_broken_parts` (seven), `sub_4410C8` (four). Refuses what
//! `fill_race_slot` refuses (before anything: the bit is cleared after).
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::{BadInput, Refused};
use crate::physics::fill_race_slot::{fill_race_slot, g, p};
use rc2k_core::physics::parts::{part_number, take_bit};

const BITS: usize = 0x71A270; // g_car + 0x298

/// The Rust version: bit `bit` (cl) of the car's parts taken and its part put in a race
/// slot. The registers the original leaves are read by none of its callers.
// Converted: `rc2k_core::physics::parts::{take_bit, part_number}`.
pub fn fill_slot_on_bit(bit: u32) -> Result<(), Refused> {
    let (_, cleared) = take_bit(g(BITS), bit);
    let Some(cleared) = cleared else {
        return Ok(());
    };
    let Some(part) = part_number(g(0x71B3F8), bit) else {
        return Err(BadInput("a part past 0xFF").into());
    };
    p(BITS, cleared);
    fill_race_slot(part)
}
