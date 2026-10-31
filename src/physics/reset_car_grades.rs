//! `physics::reset_car_grades` (0x440A22): resets a car block at esi (0xFE0 bytes, the
//! work copy's layout): +0x29C .. +0x367 zeroed (`rep stosd`); +0x3F4 .. +0x404 = 1.0
//! (`flt_5C5D38`); +0x448 .. +0x458 = `dword_71A00C` / 256 x 100, +0x438 .. +0x444 and
//! +0x45C = `dword_71A008` / 256 x 100 (both read from the work copy `g_car`, whatever
//! esi is: the setup's percentages, by the look of it); +0x408 .. +0x434 and +0x64 ..
//! +0x80 zeroed; +0x298 = -1. Stream A's reading: the block's grades and values.
//!
//! In:  esi the block
//! Out: eax 0, ecx 0, edi esi + 0x368; the rest kept; no flags read
//!
//! Callers: `physics::reset_current_car` (esi `g_car`), and unreferenced code at 0x43FDDF
//! that runs it over every car before `race::grade_all_cars`.

use crate::race::numbers_store::NUMBERS;
use rc2k_core::physics::grades::{percentage, reset_writes};

fn put(a: u32, v: u32) {
    crate::pointers::write_or_end::<u32>(a, v, "physics::reset_car_grades's operand");
}

/// The Rust core: the block at `b`, the two settings read at `s0c`, `s08` (the same at
/// any precision).
// Converted: `rc2k_core::physics::grades::{percentage, reset_writes}`.
pub fn reset_grades_at(b: u32, s0c: &crate::session::Dword, s08: &crate::session::Dword) {
    let (k256, k100) = (NUMBERS.k256.get(), NUMBERS.hundred.get());
    let a = percentage(s0c.get() as i32, k256, k100);
    let c = percentage(s08.get() as i32, k256, k100);
    for (offset, v) in reset_writes(NUMBERS.one.get(), a, c) {
        put(b + offset, v);
    }
}
