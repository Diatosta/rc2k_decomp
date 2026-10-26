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
//! Caller: `sub_43B635` (0x43B95C), every physics step. Refuses what `queue_notice` refuses
//! (everything written put back).
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::Refused;
use crate::math::x87::from_int;
use crate::physics::car_store::CAR;
use crate::physics::queue_notice::queue_notice;
use crate::physics::store::DAMAGE;
use rc2k_core::physics::damage_notices::{LEVELS, damage_notices as damage_notices_core};

/// `unk_71A284` .. `unk_71A33C`: the gauges, dwords from `CAR.brake_heat` on (the store
/// types only the first two).
fn gauge(k: usize) -> u32 {
    crate::pointers::read_or_end::<u32>(
        CAR.brake_heat[0].addr() + (4 * k) as u32,
        "physics::damage_notices's operand",
    )
}

// Converted: `rc2k_core::physics::damage_notices::damage_notices`.
fn steps() -> Result<(), Refused> {
    let mut levels: [u32; LEVELS] = std::array::from_fn(|k| CAR.damage_levels[k].get());
    let result = damage_notices_core(
        &mut levels,
        |k| f32::from_bits(gauge(k)),
        |t| DAMAGE.scratch.set_bits(from_int(t)),
        |q| queue_notice(q.notice, q.rank, gauge(q.gauge)),
    );
    for (k, &v) in levels.iter().enumerate() {
        CAR.damage_levels[k].set(v);
    }
    result
}

/// The Rust version. The registers the original leaves are read by none of its callers.
pub fn damage_notices() -> Result<(), Refused> {
    steps()
}
