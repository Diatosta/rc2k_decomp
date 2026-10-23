//! `system::packed_local_time` (0x410F99): the local date and time to the minute, packed
//! into one dword: year, 4 bits of month, 5 of day, 5 of hour, 6 of minute.
//!
//! In:  nothing
//! Out: eax = the stamp; everything else preserved
//! `GetLocalTime` fills the game's `SystemTime` (0x518988), which stays set. Six callers: the
//! save files carry this stamp.
//! The clock moves, so shadow mode and the self-test call the Rust version before and after
//! the original and accept either answer.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::{Regs, game_ptr, rva};
use windows_sys::Win32::Foundation::SYSTEMTIME;
use windows_sys::Win32::System::SystemInformation::GetLocalTime;

// ---- game data ----

const SYSTEM_TIME_RVA: usize = rva(0x518988); // SystemTime

fn system_time() -> *mut SYSTEMTIME {
    game_ptr(SYSTEM_TIME_RVA)
}

// ---- Rust version ----

pub fn pack(t: &SYSTEMTIME) -> u32 {
    // Converted: `rc2k_core::system::time::pack`.
    rc2k_core::system::time::pack(t.wYear, t.wMonth, t.wDay, t.wHour, t.wMinute)
}

pub unsafe fn packed_local_time() -> u32 {
    unsafe {
        GetLocalTime(system_time());
        pack(&system_time().read_unaligned())
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let first = unsafe { packed_local_time() };
    r.eax = first;
    Ok(())
}
