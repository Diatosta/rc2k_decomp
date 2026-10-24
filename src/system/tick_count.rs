//! `system::tick_count` (0x411024): the game's millisecond clock: `GetTickCount`, unless the
//! override switch `dword_5C2CD4` is 1, in which case it is `dword_5C2CD0`.
//!
//! In:  nothing
//! Out: eax = milliseconds; everything else preserved
//! 110 callers. What sets the override has not been read.
//! The clock moves, so shadow mode and the self-test call the Rust version before and after
//! the original and accept anything in between.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::{game_read, rva};
use windows_sys::Win32::System::SystemInformation::GetTickCount;

// ---- game data ----

const OVERRIDE_ON_RVA: usize = rva(0x5C2CD4); // dword_5C2CD4
const OVERRIDE_RVA: usize = rva(0x5C2CD0); // dword_5C2CD0

// ---- Rust version ----

pub fn tick_count() -> u32 {
    unsafe {
        // The original asks Windows first in both cases.
        let now = GetTickCount();
        // Converted: `rc2k_core::system::time::clock`.
        rc2k_core::system::time::clock(now, game_read(OVERRIDE_ON_RVA), || game_read(OVERRIDE_RVA))
    }
}
