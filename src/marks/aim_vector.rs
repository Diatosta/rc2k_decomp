//! `marks::aim_vector` (0x4C1294): for each of x, y, z `dword_769306` .. =
//! -`flt_7692B2` .. less `flt_76927A` .. plus `flt_7692CE` .. plus `flt_7692EA` ..; then
//! that vector made of length 1 into `unk_7692FA` .., its length (`fst`) in
//! `dword_7692F6` (`x87::unit_and_length`).
//!
//! In:  nothing
//! Out: nothing (no caller reads a register it leaves)
//!
//! Caller: `sub_4C1270`. Self-test: the twelve inputs by lot, compared on the seven
//! results and the registers; put back afterwards.
//!
//! Converted 2026-10-21: the hook's Rust version is `aim_vector_plain`, through
//! `rc2k_core::marks::view::{aim, unit_and_length}` (plain floats; the x87 version stays
//! for the Rust callers in unconverted code). Shadow mode and the self-test compare within
//! `service::floats`' tolerances (the self-test at both test precisions, exact at double).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::{game_read, game_write, rva};
use crate::math::x87;

/// Per axis: (negated, less, plus, plus).
const TERMS: [[usize; 4]; 3] = [
    [rva(0x7692B2), rva(0x76927A), rva(0x7692CE), rva(0x7692EA)],
    [rva(0x7692B6), rva(0x76927E), rva(0x7692D2), rva(0x7692EE)],
    [rva(0x7692BA), rva(0x769282), rva(0x7692D6), rva(0x7692F2)],
];
const VECTOR_RVA: usize = rva(0x769306); // dword_769306 .. dword_76930E
const UNIT_RVA: usize = rva(0x7692FA); // unk_7692FA
const LENGTH_RVA: usize = rva(0x7692F6); // dword_7692F6

/// The Rust version (see the header).
pub unsafe fn aim_vector() {
    unsafe {
        let v: [u32; 3] = std::array::from_fn(|k| {
            let t = TERMS[k].map(|a| game_read::<u32>(a));
            x87::negated_less_plus_plus(t[0], t[1], t[2], t[3])
        });
        game_write(VECTOR_RVA, v);
        let (unit, length) = x87::unit_and_length(v);
        game_write(LENGTH_RVA, length);
        game_write(UNIT_RVA, unit);
    }
}
