//! `physics::hit_part_checks` (0x441046): +0x284 (`unk_71A25C`) 1; with bits 5, 6, 1, 2
//! of +0x298 all set -1; with bits 7, 8, 3, 4 also set 0 (a reading: which parts are
//! left); then, with +0x26C 0, `physics::drop_hit_parts` and `physics::sound_hit_parts`.
//!
//! In:  nothing
//! Out: as the calls leave them; CF as the last `bt` or the calls leave it (no caller
//!      reads it: the next is a call)
//!
//! Caller: `sub_43B635` (0x43B957), every physics step.
//! Refuses what the calls refuse; what was written is put back.
//! Self-test: the original and the Rust version on the game's variables (saved and put
//! back): bit patterns, the switch, hits by lot.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::Refused;
use crate::physics::drop_hit_parts::{drop_hit_parts, g, p};
use crate::physics::sound_hit_parts::sound_hit_parts;
use rc2k_core::physics::parts::parts_left;

/// The Rust version. The registers the original leaves are read by none of its callers.
// Converted: `rc2k_core::physics::parts::parts_left`.
pub fn hit_part_checks() -> Result<(), Refused> {
    let left = parts_left(g(0x71A270));
    p(0x71A25C, left);
    if left != 0 || g(0x71A244) != 0 {
        return Ok(());
    }
    drop_hit_parts()?;
    sound_hit_parts()
}
