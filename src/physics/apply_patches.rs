//! Works the `patch` settings of `var\ral.cfg` into the physics' constants. Runs once: it does
//! nothing when `patches_applied` is not 0, and sets it to 1.
//!
//! - twelve settings are held to their ranges (`patches::RANGES`);
//! - `gravity` = 9.8; twelve 0x30-byte records get 1.0, 0.9 and 1.7 (1.2 in the fourth and the
//!   eleventh) in their first three floats; the drag of surfaces 7 and 8 = 10.0;
//! - if the drawing-distance setting is above 100, `RaceWork.patch_factors[1]` = it / 100;
//! - only with `patch_switch` set: the ten percent settings become `patch_factors`; `gravity`
//!   times the first; every record's three floats times one of six factors chosen by the record
//!   (`patches::BY_RECORD`: tarmac, gravel, mud, snow or grass grip) and times the fifth, fourth
//!   and third; with the sticky-grass setting at 0, the drag of surfaces 7 and 8 = 0.
//!
//! The setting names (`fizpatch`, `patchgravity`, `patchbrakegrip`, ...) are read from the
//! config's keys; the meanings are a reading, not seen in a race.
//!
//! Callers: `system::load_game_data`, and two that copy thirteen settings in from
//! `kept_patch_switches`, clear the switch and call it again. The arithmetic is plain floats
//! (`rc2k_core::physics::patches`): each figure is one correctly rounded quotient or product at
//! the game's double and extended precision.

use crate::memory::session::PlainFloat;
use crate::physics::tables_store::PhysicsTables;
use crate::physics::tables_store::Surface;
use crate::race::numbers_store::Numbers;
use crate::race::work_store::RaceWork;
use rc2k_core::physics::patches::{Patches, apply_patches as apply_patches_core};

/// The twelve ground records patched: `PHYSICS.surfaces[1 ..= 12]`.
fn records(tables: &PhysicsTables) -> &[Surface] {
    &tables.surfaces[1..13]
}

/// A record's three grips: slide, acceleration, brake.
fn grips(r: &Surface) -> [&PlainFloat; 3] {
    [&r.slide_grip, &r.accel_grip, &r.brake_grip]
}

/// The two drags (10.0) the sticky grass setting keeps: grass, records 7 and 8.
fn tens(tables: &PhysicsTables) -> [&PlainFloat; 2] {
    [&tables.surfaces[7].drag, &tables.surfaces[8].drag]
}

/// The patch settings and the figures they act on, as bits.
fn load(tables: &PhysicsTables, work: &RaceWork) -> Patches {
    let w = &work;
    Patches {
        switch: w.patch_switch().get(),
        settings: std::array::from_fn(|n| w.patch_settings()[n].get()),
        factors: std::array::from_fn(|n| w.patch_factors[n].bits()),
        applied: w.patches_applied.get(),
        gravity: tables.gravity.bits(),
        records: std::array::from_fn(|r| grips(&records(tables)[r]).map(PlainFloat::bits)),
        tens: tens(tables).map(PlainFloat::bits),
    }
}

/// The places that changed, written back.
fn store(tables: &PhysicsTables, work: &RaceWork, before: &Patches, after: &Patches) {
    let w = &work;
    if before.applied != after.applied {
        w.patches_applied.set(after.applied);
    }
    for n in 0..12 {
        if before.settings[n] != after.settings[n] {
            w.patch_settings()[n].set(after.settings[n]);
        }
    }
    let put = |f: &dyn crate::memory::session::Cell32, old: u32, new: u32| {
        if old != new {
            f.set_bits(new);
        }
    };
    put(&tables.gravity, before.gravity, after.gravity);
    for (r, record) in records(tables).iter().enumerate() {
        for (n, f) in grips(record).into_iter().enumerate() {
            put(f, before.records[r][n], after.records[r][n]);
        }
    }
    for (k, f) in tens(tables).into_iter().enumerate() {
        put(f, before.tens[k], after.tens[k]);
    }
    for n in 0..10 {
        put(&w.patch_factors[n], before.factors[n], after.factors[n]);
    }
}

/// Applies the patches at the game's double and extended precision: each figure is one
/// quotient, or a product whose first step is exact.
// Core: `rc2k_core::physics::patches::apply_patches`.
pub fn apply_patches(tables: &PhysicsTables, numbers: &Numbers, work: &RaceWork) {
    let before = load(tables, work);
    if before.applied != 0 {
        return;
    }
    let mut after = before;
    apply_patches_core(&mut after, numbers.hundred.get());
    store(tables, work, &before, &after);
}
