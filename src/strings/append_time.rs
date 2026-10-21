//! `strings::append_time` (0x401A39): appends a time given in hundredths of a second, as
//! "MM:SS.cc" or "HH:MM:SS.cc".
//!
//! In:  eax = hundredths of a second, ebx = cursor
//! Out: ebx = new cursor; everything else preserved
//! The hours field is written from one hour upwards, or always when `g_timeShowHours` is 1.

use crate::framework::{game_read, rva};

// ---- game data ----

const SHOW_HOURS_RVA: usize = rva(0x4E0148); // g_timeShowHours: 1 = always write the hours

pub fn show_hours() -> bool {
    unsafe { game_read::<u32>(SHOW_HOURS_RVA) == 1 }
}

// ---- Rust version ----

pub fn append_time(hundredths: u32, show_hours: bool) -> Vec<u8> {
    rc2k_core::strings::format::time(hundredths, show_hours)
}
