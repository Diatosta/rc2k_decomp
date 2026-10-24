//! `marks::emit_particle_11` (0x4CC35A, a chunk of `marks::emit_wheel_particles` entered
//! by `jz` from it and from 0x4CA215, left by `popa; retn`): one ring entry of kind 0xB;
//! `marks::emit_particle_5`'s code on its own variables (`dword_93A262`, `flt_93A266` ..
//! `flt_93A286`), a life of 2000, and made only when the wheel's |+0x18| rounded to an
//! integer (`fist`, the x87's rounding) is over 1 (kind 5: when it is not 0).
//!
//! In:  edi the wheel record, esi the car object
//! Out: every register kept
//!
//! Refuses as `emit_particle_5`. Self-test: as `emit_particle_5`'s.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::marks::emit_particle_5::{JitterSet, emit_particle_5_with};
use crate::marks::store::Slot;

/// Kind 0xB's variables.
pub const ELEVENTH: JitterSet = JitterSet {
    kind: 0xB,
    over_one: true,
    place_random: Slot::at(0x93A262),
    motion_k: [Slot::at(0x93A266), Slot::at(0x93A26A), Slot::at(0x93A272)],
    lift: Slot::at(0x93A26E),
    spread: Slot::at(0x93A276),
    roll_k: [Slot::at(0x93A27A), Slot::at(0x93A27E), Slot::at(0x93A282)],
    spin: Slot::at(0x93A286),
    life: 0x7D0,
};

/// The Rust version (see the header).
pub unsafe fn emit_particle_11(wheel: u32, object: u32) -> Result<(), Refused> {
    unsafe { emit_particle_5_with(&ELEVENTH, wheel, object) }
}
