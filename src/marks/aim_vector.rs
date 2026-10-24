//! `marks::aim_vector` (0x4C1294): for each of x, y, z `dword_769306` .. =
//! -`flt_7692B2` .. less `flt_76927A` .. plus `flt_7692CE` .. plus `flt_7692EA` ..; then
//! that vector made of length 1 into `unk_7692FA` .., its length (`fst`) in
//! `dword_7692F6` (`x87::unit_and_length`).
//!
//! In:  nothing
//! Out: nothing (no caller reads a register it leaves)
//!
//! Caller: `sub_4C1270`.
//!
//! the x87 version stays for the Rust callers in unconverted code).

use crate::marks::store::{Slot, game_read, game_write};
use crate::math::x87;

/// Per axis: (negated, less, plus, plus).
const TERMS: [[Slot; 4]; 3] = [
    [
        Slot::at(0x7692B2),
        Slot::at(0x76927A),
        Slot::at(0x7692CE),
        Slot::at(0x7692EA),
    ],
    [
        Slot::at(0x7692B6),
        Slot::at(0x76927E),
        Slot::at(0x7692D2),
        Slot::at(0x7692EE),
    ],
    [
        Slot::at(0x7692BA),
        Slot::at(0x769282),
        Slot::at(0x7692D6),
        Slot::at(0x7692F2),
    ],
];
const VECTOR_RVA: Slot = Slot::at(0x769306); // dword_769306 .. dword_76930E
const UNIT_RVA: Slot = Slot::at(0x7692FA); // unk_7692FA
const LENGTH_RVA: Slot = Slot::at(0x7692F6); // dword_7692F6

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
