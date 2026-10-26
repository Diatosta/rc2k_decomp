//! `system::packed_local_time` (0x410F99): the local date and time to the minute, packed
//! into one dword: year, 4 bits of month, 5 of day, 5 of hour, 6 of minute.
//!
//! In:  nothing
//! Out: eax = the stamp; everything else preserved
//! `GetLocalTime` fills the game's `SystemTime` (0x518988, a local here: nothing else read
//! it). Six callers: the
//! save files carry this stamp.
//! The clock moves, so shadow mode and the self-test call the Rust version before and after
//! the original and accept either answer.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

// ---- Rust version ----

pub fn pack(t: &rc2k_core::system::time::LocalTime) -> u32 {
    // Converted: `rc2k_core::system::time::pack`.
    rc2k_core::system::time::pack(t.year, t.month, t.day, t.hour, t.minute)
}

pub fn packed_local_time() -> u32 {
    pack(&crate::os::local_time())
}
