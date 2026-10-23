//! `marks::place_view_2` (0x4C2AA5): `marks::place_view` on other variables (the point
//! (0, 0, 0x80000), the matrix at `dword_8D216C`, the place `dword_8D2160`, the height
//! `flt_8DB53C`, out at `dword_8D2214`, the copy at `dword_8D2220`), then `dword_8D20C0`
//! .. `dword_8D20DC` 0 and the four floats at `xmmword_8D20E0` 0x80000.
//!
//! In:  nothing
//! Out: eax 0; the rest as `scene::transform_int_points` leaves them
//!
//! See `marks/place_view.rs`; no variable shared with it. One caller, `sub_4C25F6`.
//! Self-test: as `place_view`'s, the two blocks also watched.
//!
//! Converted 2026-10-21: the hook's Rust version is `place_view_2_plain`, on
//! `marks::place_view`'s converted body (the x87 version stays for the Rust callers in
//! unconverted code); compared as `place_view`.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_read, game_write, rva};
use crate::marks::place_view::{ViewPlace, place_view_with, place_view_with_plain};
use crate::math::x87;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;

const SECOND: ViewPlace = ViewPlace {
    point: rva(0x8D2208), // dword_8D2208
    start_z: 0x8_0000,
    place: rva(0x8D2160),       // dword_8D2160
    matrix: rva(0x8D216C),      // dword_8D216C
    out: rva(0x8D2214),         // dword_8D2214
    height: rva(0x8DB53C),      // flt_8DB53C
    matrix_copy: rva(0x8D2220), // dword_8D2220
};
const CLEARED_RVA: usize = rva(0x8D20C0); // dword_8D20C0 .. dword_8D20DC
const FILLED_RVA: usize = rva(0x8D20E0); // xmmword_8D20E0: four floats

/// The Rust version (see the header).
pub unsafe fn place_view_2(entry: &Regs) -> Result<Regs, Refused> {
    unsafe {
        let mut g = place_view_with(entry, &SECOND)?;
        g.eax = 0;
        game_write(CLEARED_RVA, [0u32; 8]);
        let z = x87::from_int(game_read(SECOND.point + 8));
        game_write(FILLED_RVA, [z; 4]);
        Ok(g)
    }
}

/// The converted version (plain floats): the hook's.
// Converted: `rc2k_core::marks::view::view_place`
pub unsafe fn place_view_2_plain(entry: &Regs) -> Result<Regs, Refused> {
    unsafe {
        let mut g = place_view_with_plain(entry, &SECOND)?;
        g.eax = 0;
        game_write(CLEARED_RVA, [0u32; 8]);
        // The point is (0, 0, 0x80000) again: the float exact at any rounding.
        let z = x87::from_int(game_read(SECOND.point + 8));
        game_write(FILLED_RVA, [z; 4]);
        Ok(g)
    }
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { place_view_2_plain(r) }?))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    crate::service::floats::floats_entry(r, &rust)
}
