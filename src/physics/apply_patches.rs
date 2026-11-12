//! `physics::apply_patches` (0x4648BD): the physics' constants, with the `patch` settings
//! worked into them. Once.
//!
//! Nothing if `g_patchesApplied` (0x71C72C) is not 0; else that becomes 1 and:
//! - twelve settings are held to their ranges (`patches::RANGES`; the comparisons look at
//!   the sign of the difference only, which is kept);
//! - `gravity` = 9.8; twelve records of 0x30 bytes from 0x6139AC on get 1.0, 0.9 and
//!   1.7 (1.2 in the fourth and the eleventh) in their first three floats; `PhysicsTables.surfaces[7].drag`
//!   and `PhysicsTables.surfaces[8].drag` = 10.0;
//! - if the setting at 0x71C6E0 is above 100, `RaceWork.patch_factors[1]` = it / 100;
//! - only if the settings' switch `patch_switch` is not 0: the ten floats from
//!   `patch_factors` on = the ten settings from 0x71C6DC on, each / 100; `gravity` times
//!   the first; every record's three floats times one of six of them, by the record
//!   (`patches::BY_RECORD`), and times the fifth, the fourth and the third; and with the setting at
//!   0x71C6D8 at 0, `PhysicsTables.surfaces[7].drag` and `PhysicsTables.surfaces[8].drag` = 0.
//!
//! The settings' keys in `var\ral.cfg` say what they are, from 0x71C6D0 on: `fizpatch`
//! (the switch), `patchmaxdrawdistance`, `patchstickygrass`, `patchgravity`,
//! `patchdamages`, `patchbrakegrip`, `patchaccelgrip`, `patchslidegrip`, `patchgripgrass`,
//! `patchgriptarmac`, `patchgripsnow`, `patchgripmud`, `patchgripgravel`, the last ten in
//! percent. So `gravity` is gravity, and a record is a kind of ground with its slide,
//! acceleration and brake grip: by `BY_RECORD` tarmac, gravel, mud, snow, grass, mud,
//! grass, grass, snow, grass, snow, gravel. A reading of the names, not seen in a race.
//! Callers: `system::load_game_data`, and two that copy thirteen settings in from
//! `kept_patch_switches`, clear the switch and call it again. The arithmetic is plain floats
//! (`rc2k_core::physics::patches`): each figure is one correctly rounded quotient or
//! product at the game's double and extended precision.

use crate::physics::tables_store::PhysicsTables;
use crate::physics::tables_store::Surface;
use crate::race::numbers_store::Numbers;
use crate::race::work_store::RaceWork;
use crate::session::PlainFloat;
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

/// The game's patch state.
fn load(tables: &PhysicsTables, work: &RaceWork) -> Patches {
    let w = &work;
    Patches {
        switch: w.patch_switch.get(),
        settings: std::array::from_fn(|n| w.patch_settings[n].get()),
        factors: std::array::from_fn(|n| w.patch_factors[n].bits()),
        applied: w.patches_applied.get(),
        gravity: tables.gravity.bits(),
        records: std::array::from_fn(|r| grips(&records(tables)[r]).map(PlainFloat::bits)),
        tens: tens(tables).map(PlainFloat::bits),
    }
}

/// The places that changed written back (the original writes only these, or the same
/// value again).
fn store(tables: &PhysicsTables, work: &RaceWork, before: &Patches, after: &Patches) {
    let w = &work;
    if before.applied != after.applied {
        w.patches_applied.set(after.applied);
    }
    for n in 0..12 {
        if before.settings[n] != after.settings[n] {
            w.patch_settings[n].set(after.settings[n]);
        }
    }
    let put = |f: &dyn crate::session::Cell32, old: u32, new: u32| {
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

/// Applies the patches (bit-identical to the original at the game's double and extended
/// precision: each figure is one quotient, or a product whose first step is exact).
// Converted: `rc2k_core::physics::patches::apply_patches`.
pub fn apply_patches(tables: &PhysicsTables, numbers: &Numbers, work: &RaceWork) {
    let before = load(tables, work);
    if before.applied != 0 {
        return;
    }
    let mut after = before;
    apply_patches_core(&mut after, numbers.hundred.get());
    store(tables, work, &before, &after);
}
