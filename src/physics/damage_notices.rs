//! `physics::damage_notices` (0x4B0241): the work car's damage warnings. For each damage
//! gauge (floats of the work copy, `unk_71A284` .. `unk_71A33C`) a level (`unk_71A398`,
//! `unk_71A340` ..) moved by thresholds, in the original's order of 82 steps (`STEPS`,
//! read from the disassembly): a step for level `from` puts the threshold (a whole number)
//! into `flt_75B36C` (`fild` / `fstp`, every step) and compares the gauge with it
//! (`fld; fcomp`, below or unordered as C0): rising steps, not below, set the level to
//! `to` and queue a notice (`physics::queue_notice`: esi the notice, eax the gauge, ebx
//! the rank); falling steps (the hysteresis of some gauges), below, set it back.
//!
//! In:  nothing
//! Out: eax, ebx, esi the last notice's, edx as `queue_notice` leaves it (kept when none);
//!      the rest kept; no flags read (the caller goes on)
//!
//! Caller: `sub_43B635` (0x43B95C), every physics step.
//! Refuses what `queue_notice` refuses (everything written put back).
//! Self-test: the original and the Rust version on the game's variables (saved and put
//! back): gauges and levels by lot, notices on and off (`dword_71A1F8` bit 2), the queue
//! empty and part full, under three precisions.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::math::x87::from_int;
use crate::physics::flow::{g, p};
use crate::physics::queue_notice::queue_notice;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use rc2k_core::physics::damage_notices::{LEVELS, damage_notices as damage_notices_core};

/// `flt_75B36C`: the threshold being compared.
const LIMIT: usize = 0x75B36C;

const LEVELS_AT: usize = 0x71A340; // unk_71A340: the levels
const GAUGES_AT: usize = 0x71A284; // unk_71A284: the gauges

// Converted: `rc2k_core::physics::damage_notices::damage_notices`.
fn steps(r: &mut Regs) -> Result<(), Refused> {
    let mut levels: [u32; LEVELS] = std::array::from_fn(|k| g(LEVELS_AT + 4 * k));
    let result = damage_notices_core(
        &mut levels,
        |k| f32::from_bits(g(GAUGES_AT + 4 * k)),
        |t| p(LIMIT, from_int(t)),
        |q| {
            r.esi = q.notice;
            r.eax = g(GAUGES_AT + 4 * q.gauge);
            r.ebx = q.rank;
            unsafe { queue_notice(r) }?.apply(r);
            Ok::<(), Refused>(())
        },
    );
    for (k, &v) in levels.iter().enumerate() {
        p(LEVELS_AT + 4 * k, v);
    }
    result
}

/// The Rust version.
pub unsafe fn damage_notices(entry: &Regs) -> Result<Left, Refused> {
    {
        let mut r = *entry;
        steps(&mut r)?;
        Ok(left_of(&r))
    }
}
