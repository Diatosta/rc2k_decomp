//! `physics::allowed_gear` (0x4651B5): eax kept or made 0. 0 when `g_car` +0x2B8
//! (`unk_71A290`) holds the bits of `flt_5C5D14` (100.0, compared as a dword); for eax
//! 1 or more (by the sign of eax - 1) also 0 when `physics::gear_bit` of it gives CF;
//! then 0 unless +0x3F4 (`unk_71A3CC`) is set. A reading: the gear asked for, unless the
//! gearbox refuses it.
//!
//! In:  eax the gear
//! Out: eax the gear or 0; the rest kept (ebx and ecx pushed around); no flags read
//!      (the caller stores eax)
//!
//! Caller: `sub_43C1E3` (0x43C1FD).
//! Refuses what `gear_bit` refuses (a bit past 31).
//! Self-test: a copy of the original (its two words the test's, its call to
//! `gear_bit`'s test copy on a test dword) against the core: gears -1 .. 9, the words'
//! settings, several bit patterns.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{game_addr, game_read, rva};
use crate::physics::gear_bit::bits_at;

unsafe fn at(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

/// The Rust version on the word +0x2B8, the word +0x3F4 and the gear bits at these places.
// Converted: `rc2k_core::physics::gearbox::allowed_gear`.
pub unsafe fn allowed_gear_at(locked: u32, open: u32, bits: u32, eax: u32) -> Result<u32, Refused> {
    unsafe {
        let locked = at(locked) == game_read::<u32>(rva(0x5C5D14));
        Ok(rc2k_core::physics::gearbox::allowed_gear(
            locked,
            eax,
            || bits_at(bits),
            || at(open),
        )?)
    }
}

/// The Rust version.
pub unsafe fn allowed_gear(eax: u32) -> Result<u32, Refused> {
    let g = |a: usize| game_addr(rva(a)) as u32;
    unsafe { allowed_gear_at(g(0x71A290), g(0x71A3CC), g(0x71A40C), eax) }
}
