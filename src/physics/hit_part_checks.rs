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
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_addr, rva};
use crate::physics::drop_hit_parts::{drop_hit_parts_in, g, p};
use crate::physics::fill_race_slot::regions as slot_regions;
use crate::physics::sound_hit_parts::sound_hit_parts_in;
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, restore, state_entry};
use rc2k_core::physics::parts::parts_left;

/// The steps on `r`.
// Converted: `rc2k_core::physics::parts::parts_left`.
pub fn hit_part_checks_in(r: &mut Regs) -> Result<(), Refused> {
    let left = parts_left(g(0x71A270));
    p(0x71A25C, left);
    if left != 0 || g(0x71A244) != 0 {
        return Ok(());
    }
    drop_hit_parts_in(r)?;
    sound_hit_parts_in(r)
}

fn regions() -> Vec<Region> {
    let a = |x: usize| game_addr(rva(x)) as u32;
    let mut r = slot_regions();
    r.extend([(a(0x71A25C), 4), (a(0x71A2D0), 0x10), (a(0x71A32C), 0x10)]);
    let o = g(0x512A10 + 4 * (g(0x71B3F8) & 7) as usize);
    if o != 0 {
        r.extend([(o + 0x10, 4), (o + 0x214, 0x14)]);
    }
    r
}

/// The Rust version.
pub unsafe fn hit_part_checks(entry: &Regs) -> Result<Left, Refused> {
    let all = regions();
    let saved = capture(&all);
    let mut r = *entry;
    match hit_part_checks_in(&mut r) {
        Ok(()) => Ok(crate::race::stage_prefs::left_of(&r)),
        Err(e) => {
            restore(&all, &saved);
            Err(e)
        }
    }
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { hit_part_checks(r) }
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
