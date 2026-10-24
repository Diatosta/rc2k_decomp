//! `marks::place_view_2` (0x4C2AA5): `marks::place_view` on other variables (the point
//! (0, 0, 0x80000), the matrix at `dword_8D216C`, the place `dword_8D2160`, the height
//! `flt_8DB53C`, out at `dword_8D2214`, the copy at `dword_8D2220`), then `dword_8D20C0`
//! .. `dword_8D20DC` 0 and the four floats at `xmmword_8D20E0` 0x80000.
//!
//! In:  nothing
//! Out: nothing (its caller reads no register after it)
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
use crate::marks::place_view::{ViewPlace, place_view_with};
use crate::marks::store::{Slot, game_read, game_write};
use crate::math::x87;

const SECOND: ViewPlace = ViewPlace {
    point: Slot::at(0x8D2208), // dword_8D2208
    start_z: 0x8_0000,
    place: Slot::at(0x8D2160),       // dword_8D2160
    matrix: Slot::at(0x8D216C),      // dword_8D216C
    out: Slot::at(0x8D2214),         // dword_8D2214
    height: Slot::at(0x8DB53C),      // flt_8DB53C
    matrix_copy: Slot::at(0x8D2220), // dword_8D2220
};
const CLEARED_RVA: Slot = Slot::at(0x8D20C0); // dword_8D20C0 .. dword_8D20DC
const FILLED_RVA: Slot = Slot::at(0x8D20E0); // xmmword_8D20E0: four floats

/// The Rust version (see the header).
pub unsafe fn place_view_2() -> Result<(), Refused> {
    unsafe {
        place_view_with(&SECOND)?;
        game_write(CLEARED_RVA, [0u32; 8]);
        let z = x87::from_int(game_read(SECOND.point + 8));
        game_write(FILLED_RVA, [z; 4]);
        Ok(())
    }
}
