//! `sound::merge_spot_voices` (0x40E809): of the voices heard with one sound, the
//! nearest kept and (re)started, moved to the point between it and the next nearest
//! that is nearest the listener; all the others silenced.
//!
//! In:  eax the sound
//! Out: ebp the sound, ecx 0; ebx the nearest (or the end of the voices), eax and edx
//!      as the original leaves them (see the code); esi, edi preserved
//! The pickers' work (`dword_517F40` .. `dword_517F50`) cleared (distances to -1). For
//! each voice heard with the sound: a playing one noted in `dword_517F40`; its distance
//! x 1000 as a whole number (`fistp` through `g_soundScratch`) compared (unsigned) with
//! the nearest's and the next's, which move down; the voice silenced (+4 and +0x28 0).
//! The nearest, if any, gets +0x24 and +4 1; with a next as well, `math::nearest_on_segment`
//! of their places and the listener (0, 0, 0) gives the nearest's new place.
//! Six callers, `sound::merge_spot_voices_all`.
//! Shadow mode: both (nothing sent), the voices and variables compared.
//! Self-test: a copy whose call goes to a stand-in (which runs the Rust
//! `nearest_on_segment`), against the Rust version, on the game's voices (put back).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_addr, rva};
use crate::physics::flow::{base, x87_run};
use crate::service::harness::Region;
use crate::sound::spot_voices::{WORK, g, p, voice, voice_regions};
use crate::sound::turn_parts::{Part, SCRATCH, turn_entry};

pub const NEAREST_ON_SEGMENT: u32 = 0x4633DE;
const ENDS: usize = 0x71C384;
const RESULT: usize = 0x71C3A8;
pub const PLAYING: usize = WORK;
pub const BEST_D: usize = WORK + 4;
pub const BEST: usize = WORK + 8;
pub const NEXT_D: usize = WORK + 12;
pub const NEXT: usize = WORK + 16;
/// `mov scratch, 1000.0; fld [v+0x40]; fmul scratch; fistp scratch`.
pub fn distance_1000(v: u32) -> u32 {
    p(SCRATCH, 0x447A_0000);
    x87_run!(
        "fld dword ptr [{bx} + 0x40]",
        "fmul dword ptr [{b} + 0x1183CC]",
        "fistp dword ptr [{b} + 0x1183CC]";
        bx = v, b = base()
    );
    g(SCRATCH)
}

/// The pickers' work cleared.
pub fn clear_work() {
    for (at, v) in [
        (PLAYING, 0),
        (BEST_D, u32::MAX),
        (BEST, 0),
        (NEXT_D, u32::MAX),
        (NEXT, 0),
    ] {
        p(at, v);
    }
}

// ---- Rust version ----

/// Converted: `rc2k_core::sound::spot_voices::merge` and `merge_between`.
pub fn merge_spot_voices_in(c: &mut Regs, part: Part) -> Result<(), Refused> {
    use crate::sound::spot_voices::{load_voices, pick_regs, store_picks, store_voices};
    use rc2k_core::sound::spot_voices as core;
    let mut voices = load_voices();
    let (picks, trail) = core::merge(&mut voices, c.eax);
    store_voices(&voices);
    store_picks(&picks, &trail);
    *c = pick_regs(c, &trail);
    let Some(nearest) = picks.nearest else {
        return Ok(());
    };
    c.ebx = voice(nearest as u32);
    let Some(next) = picks.next else {
        return Ok(());
    };
    let (a, b) = (voices[nearest].place, voices[next].place);
    for m in 0..3 {
        p(ENDS + 4 * m, a[m]);
        p(ENDS + 12 + 4 * m, b[m]);
        p(ENDS + 24 + 4 * m, 0);
    }
    (c.eax, c.ebx) = (b[2], voice(next as u32));
    part(NEAREST_ON_SEGMENT, c)?;
    let point = [0, 1, 2].map(|m| g(RESULT + 4 * m));
    core::merge_between(&mut voices, &picks, point);
    store_voices(&voices);
    (c.eax, c.ebx) = (point[2], voice(nearest as u32));
    Ok(())
}

pub fn merge_regions(_: &Regs) -> Vec<Region> {
    let mut all = voice_regions();
    use crate::math::nearest_on_segment::{VARS, VARS_LEN};
    all.push((game_addr(rva(VARS)) as u32, VARS_LEN));
    all
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    turn_entry(r, merge_spot_voices_in)
}
