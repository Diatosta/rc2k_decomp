//! `drive::play_replay_frame` (0x460049): while playing a replay (`dword_5C5CE0` 2): the
//! replay's frame and tenth (`dword_5C2530`, `5C2534`) into `dword_6EF7E0`, `6EF7E4`; at
//! a whole frame the frame read and its state loaded (`drive::load_replay_frame`,
//! `drive::load_replay_cars`); then the cameras (`drive::replay_cameras`).
//!
//! In:  nothing
//! Out: as the calls leave the registers; no flags read
//!
//! Caller: `drive::replay_step`.
//! Refuses what its calls refuse.
//! Self-test: the original and the Rust version with `read_replay_frame`'s test replay:
//! not playing, a whole frame, a tenth.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::physics::flow::{a, g, p};
use crate::screens::harness::Left;
use crate::service::harness::{Region, state_entry};

/// The Rust version.
// Converted: `rc2k_core::drive::race_steps::play_frame`.
pub unsafe fn play_replay_frame(entry: &Regs) -> Result<Left, Refused> {
    let mut r = *entry;
    let Some(whole) = rc2k_core::drive::race_steps::play_frame(g(0x5C5CE0), || {
        p(0x6EF7E0, g(0x5C2530));
        p(0x6EF7E4, g(0x5C2534));
        g(0x6EF7E4)
    }) else {
        return Ok(Left::nothing());
    };
    if whole {
        unsafe { crate::drive::load_replay_frame::load_replay_frame() }?.apply(&mut r);
        crate::drive::load_replay_cars::load_replay_cars().apply(&mut r);
    }
    unsafe { crate::drive::replay_cameras::replay_cameras(&r) }?.apply(&mut r);
    Ok(crate::race::stage_prefs::left_of(&r))
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { play_replay_frame(r) }
}

/// What it and its calls write.
pub fn regions() -> Vec<Region> {
    let mut all = crate::drive::read_replay_frame::regions();
    all.extend(crate::drive::replay_cameras::regions());
    all.extend([
        (a(0x6EF7E0), 8),
        (a(0x7172EC), 4 * 0xA04),
        (a(0x70F3EC), 0x3F80),
        (a(0x71BAC8), 0x3C0),
        (a(0x71BF84), 0x400),
    ]);
    all
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
