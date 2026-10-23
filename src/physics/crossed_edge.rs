//! `physics::crossed_edge` (0x45D737): when a probe point crossed a face's edge
//! (`dword_6D3B84` not -1): its three floats from `unk_71AAC8[n]` .. into `unk_71A1FC`
//! ..; `physics::restore_saved_car`; `physics::crossed_edge_contact`; `unk_71A208`
//! raised to |`flt_6D3108`|, `unk_71A20C` = 1, `unk_71A1CC` = 0x32;
//! `physics::feel_ground`.
//!
//! In:  nothing
//! Out: the registers as the calls leave them; no flags read (callers go on)
//!
//! Callers: `sub_43B635` (every physics step), `sub_43BDD1`.
//! Refuses (before anything) a probe point past 0x3F (the tables' room), and what its
//! calls refuse (what was written put back).
//! Self-test (`test_crossed`; in a race, its `feel_ground` needs the stage): the
//! original and the Rust version on the game's variables (saved and put back) with a
//! test face and scene points, paths across each edge, none crossed, under three
//! precisions.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::physics::crossed_edge_contact::{CROSSED, check_probe};
use crate::physics::flow::{g, p};
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;

/// The steps, not guarded (the probe checked before).
// Converted: `rc2k_core::physics::walls::{NONE_CROSSED, raised_hit}`.
pub fn steps(r: &mut Regs) -> Result<(), Refused> {
    let n = g(CROSSED);
    if n == rc2k_core::physics::walls::NONE_CROSSED {
        return Ok(());
    }
    r.ebx = n;
    let k = n as usize;
    p(0x71A1FC, g(0x71AAC8 + 4 * k));
    p(0x71A200, g(0x71ABC8 + 4 * k));
    p(0x71A204, g(0x71ACC8 + 4 * k));
    unsafe { crate::physics::restore_saved_car::restore_saved_car() }?.apply(r);
    crate::physics::crossed_edge_contact::steps(r)?;
    if let Some(hit) = rc2k_core::physics::walls::raised_hit(
        f32::from_bits(g(0x6D3108)),
        f32::from_bits(g(0x71A208)),
    ) {
        p(0x71A208, hit.to_bits());
    }
    p(0x71A20C, 1);
    p(0x71A1CC, 0x32);
    unsafe { crate::physics::feel_ground::feel_ground(r) }
}

/// The probe checked (-1 is none).
pub fn check() -> Result<(), Refused> {
    if g(CROSSED) == rc2k_core::physics::walls::NONE_CROSSED {
        return Ok(());
    }
    check_probe()
}

/// The Rust version.
pub unsafe fn crossed_edge(entry: &Regs) -> Result<Left, Refused> {
    check()?;
    {
        let mut r = *entry;
        steps(&mut r)?;
        Ok(left_of(&r))
    }
}
