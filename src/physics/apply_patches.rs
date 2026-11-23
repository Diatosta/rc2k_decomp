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
//! The arithmetic is plain floats (`rc2k_core::physics::patches`): each figure is one correctly
//! rounded quotient or product at the game's double and extended precision.

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
fn grips(r: &mut Surface) -> [&mut f32; 3] {
    [&mut r.slide_grip, &mut r.accel_grip, &mut r.brake_grip]
}

/// The two drags (10.0) the sticky grass setting keeps: grass, records 7 and 8.
fn tens(tables: &PhysicsTables) -> [f32; 2] {
    [tables.surfaces[7].drag, tables.surfaces[8].drag]
}

/// The patch settings and the figures they act on, as bits.
fn load(tables: &PhysicsTables, work: &RaceWork) -> Patches {
    let w = work;
    Patches {
        switch: w.patch_switch(),
        settings: std::array::from_fn(|n| w.patch_settings()[n]),
        factors: std::array::from_fn(|n| w.patch_factors[n].to_bits()),
        applied: w.patches_applied,
        gravity: tables.gravity.to_bits(),
        records: std::array::from_fn(|r| {
            let s = &records(tables)[r];
            [s.slide_grip, s.accel_grip, s.brake_grip].map(f32::to_bits)
        }),
        tens: tens(tables).map(f32::to_bits),
    }
}

/// The places that changed, written back.
fn store(tables: &mut PhysicsTables, work: &mut RaceWork, before: &Patches, after: &Patches) {
    if before.applied != after.applied {
        work.patches_applied = after.applied;
    }
    for n in 0..12 {
        if before.settings[n] != after.settings[n] {
            work.patches[1 + n] = after.settings[n];
        }
    }
    let put = |f: &mut f32, old: u32, new: u32| {
        if old != new {
            *f = f32::from_bits(new);
        }
    };
    put(&mut tables.gravity, before.gravity, after.gravity);
    for (r, record) in tables.surfaces[1..13].iter_mut().enumerate() {
        for (n, f) in grips(record).into_iter().enumerate() {
            put(f, before.records[r][n], after.records[r][n]);
        }
    }
    for (k, surface) in [7, 8].into_iter().enumerate() {
        put(
            &mut tables.surfaces[surface].drag,
            before.tens[k],
            after.tens[k],
        );
    }
    for n in 0..10 {
        if before.factors[n] != after.factors[n] {
            work.patch_factors[n] = f32::from_bits(after.factors[n]);
        }
    }
}

/// Applies the patches at the game's double and extended precision: each figure is one
/// quotient, or a product whose first step is exact.
// Core: `rc2k_core::physics::patches::apply_patches`.
pub fn apply_patches(tables: &mut PhysicsTables, numbers: &Numbers, work: &mut RaceWork) {
    let before = load(tables, work);
    if before.applied != 0 {
        return;
    }
    let mut after = before;
    apply_patches_core(&mut after, numbers.hundred.get());
    store(tables, work, &before, &after);
}
