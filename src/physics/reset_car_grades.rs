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
//! Callers: `physics::reset_current_car` (esi `g_car`), and unreferenced code at
//! 0x43FDDF that runs it over every car before `race::grade_all_cars`.
//! Self-test: a copy of the original on a test block (its two reads of the work copy
//! made the test's) against the converted version, fillings by lot at the test
//! precisions (exact at any).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::{EAX, ECX, EDI, game_read, rva};
use crate::screens::harness::Left;
use rc2k_core::physics::grades::{percentage, reset_writes};

const ONE_RVA: usize = rva(0x5C5D38); // flt_5C5D38, 1.0
const K256_RVA: usize = rva(0x5C5E2C); // flt_5C5E2C, 256.0
const K100_RVA: usize = rva(0x5C5D14); // flt_5C5D14, 100.0

unsafe fn put(a: u32, v: u32) {
    unsafe { (a as *mut u32).write_unaligned(v) }
}

/// The Rust core: the block at `b`, the two settings read at `s0c`, `s08` (the same at
/// any precision).
// Converted: `rc2k_core::physics::grades::{percentage, reset_writes}`.
pub unsafe fn reset_grades_at(b: u32, s0c: u32, s08: u32) -> Left {
    unsafe {
        let k = |a: usize| f32::from_bits(game_read(a));
        let (k256, k100) = (k(K256_RVA), k(K100_RVA));
        let a = percentage((s0c as *const i32).read_unaligned(), k256, k100);
        let c = percentage((s08 as *const i32).read_unaligned(), k256, k100);
        for (offset, v) in reset_writes(k(ONE_RVA), a, c) {
            put(b + offset, v);
        }
        Left::nothing()
            .with(EAX, 0)
            .with(ECX, 0)
            .with(EDI, b + 0x368)
    }
}
