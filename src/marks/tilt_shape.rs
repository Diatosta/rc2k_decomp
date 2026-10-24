//! `marks::tilt_shape` (0x4C1194): the sine and cosine of `g_marksShapeTilt` into
//! `flt_811BC7` / `flt_811BCB`; the point's y and z (`flt_811BE3`, `flt_811BE7`) turned by
//! them (`x87::turn_yz`); then the point `flt_811BDF` .. made of length 1 into
//! `unk_811BD3` .., its length in `dword_811BCF` (`x87::unit_and_length`).
//!
//! In:  nothing
//! Out: nothing (no caller reads a register it leaves)
//!
//! Caller: `sub_4C1148`.
//!
//! the x87 version stays for the Rust callers in unconverted code).

use crate::marks::store::{Slot, game_read, game_write};
use crate::math::x87;

const TILT_RVA: Slot = Slot::at(0x811BC3); // g_marksShapeTilt
const SINE_RVA: Slot = Slot::at(0x811BC7); // flt_811BC7
const COSINE_RVA: Slot = Slot::at(0x811BCB); // flt_811BCB
const POINT_RVA: Slot = Slot::at(0x811BDF); // flt_811BDF .. flt_811BE7
const UNIT_RVA: Slot = Slot::at(0x811BD3); // unk_811BD3
const LENGTH_RVA: Slot = Slot::at(0x811BCF); // dword_811BCF

/// The Rust version (see the header).
pub unsafe fn tilt_shape() {
    unsafe {
        let (s, c) = x87::sine_and_cosine(game_read(TILT_RVA));
        game_write(COSINE_RVA, c);
        game_write(SINE_RVA, s);
        let (y, z) = x87::turn_yz(game_read(POINT_RVA + 4), game_read(POINT_RVA + 8), s, c);
        game_write(POINT_RVA + 4, y);
        game_write(POINT_RVA + 8, z);
        let (unit, length) = x87::unit_and_length(game_read(POINT_RVA));
        game_write(LENGTH_RVA, length);
        game_write(UNIT_RVA, unit);
    }
}
