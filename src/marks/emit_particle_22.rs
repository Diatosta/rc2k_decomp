//! `marks::emit_particle_22` (0x4CE482): `marks::emit_particle_19`'s code on its own
//! variables: kind 0x16, the level the car's +0x4C4, the flags 0x33, `dword_93A4B2`,
//! `flt_93A4B6` .. `flt_93A4D2`, `dword_93A4D6` (an instruction diff: nothing else
//! differs).
//!
//! In:  esi the car object
//! Out: every register kept
//!
//! Refuses as `emit_particle_19`. Self-test: as `emit_particle_19`'s.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::rva;
use crate::marks::emit_particle_19::{TwinSet, emit_twin};

/// `emit_particle_22`'s variables.
pub const TWENTY_SECOND: TwinSet = TwinSet {
    kind: 0x16,
    level_at: 0x4C4,
    flags: 0x33,
    chance_random: rva(0x93A4B2),
    lift: rva(0x93A4B6),
    row_k: [rva(0x93A4BA), rva(0x93A4BE), rva(0x93A4C2)],
    spread: rva(0x93A4C6),
    motion_k: [rva(0x93A4CA), rva(0x93A4CE), rva(0x93A4D2)],
    spin_random: rva(0x93A4D6),
};

/// The Rust version (see the header).
pub unsafe fn emit_particle_22(object: u32) -> Result<(), Refused> {
    unsafe { emit_twin(&TWENTY_SECOND, object) }
}
