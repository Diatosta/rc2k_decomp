//! `marks::copy_first_point` (0x4C1163): `dword_769306` .. `dword_76930E` into
//! `flt_811BDF` .. `flt_811BE7`, and `dword_7692F6` into `dword_811BCF` (`push`/`pop`).
//!
//! In:  nothing
//! Out: every register kept
//!
//! Caller: `sub_4C1148`.

use crate::marks::store::{Slot, game_write};
use crate::marks::view_vectors::KEPT;

/// The Rust version (see the header).
pub fn copy_first_point() {
    for (n, to) in [Slot::at(0x811BDF), Slot::at(0x811BE3), Slot::at(0x811BE7)]
        .into_iter()
        .enumerate()
    {
        unsafe { game_write(to, KEPT.vector[n].bits()) };
    }
    unsafe { game_write(Slot::at(0x811BCF), KEPT.length.get()) };
}
