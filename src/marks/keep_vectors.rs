//! `marks::keep_vectors` (0x4C1339): twelve dwords kept (`push`/`pop`): `dword_769252` ..
//! and `dword_76924E` into `dword_76926E` .. and `dword_76926A`; `dword_764FB0` .. and
//! `dword_764FAC` into `dword_7692C2` .. and `dword_7692BE`; `dword_764F94` .. and
//! `dword_764F90` into `dword_7692A6` .. and `dword_7692A2`.
//!
//! In:  nothing
//! Out: every register kept
//!
//! Caller: `sub_4C1270`.

use crate::marks::store::{Slot, game_read, game_write};

/// (from, to), in the original's order.
const MOVES: [(Slot, Slot); 12] = [
    (Slot::at(0x769252), Slot::at(0x76926E)),
    (Slot::at(0x769256), Slot::at(0x769272)),
    (Slot::at(0x76925A), Slot::at(0x769276)),
    (Slot::at(0x76924E), Slot::at(0x76926A)),
    (Slot::at(0x764FB0), Slot::at(0x7692C2)),
    (Slot::at(0x764FB4), Slot::at(0x7692C6)),
    (Slot::at(0x764FB8), Slot::at(0x7692CA)),
    (Slot::at(0x764FAC), Slot::at(0x7692BE)),
    (Slot::at(0x764F94), Slot::at(0x7692A6)),
    (Slot::at(0x764F98), Slot::at(0x7692AA)),
    (Slot::at(0x764F9C), Slot::at(0x7692AE)),
    (Slot::at(0x764F90), Slot::at(0x7692A2)),
];

/// The Rust version (see the header).
pub unsafe fn keep_vectors() {
    for (from, to) in MOVES {
        unsafe { game_write(to, game_read::<u32>(from)) };
    }
}
