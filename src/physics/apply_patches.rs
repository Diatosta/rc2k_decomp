//! `physics::apply_patches` (0x4648BD): the physics' constants, with the `patch` settings
//! worked into them. Once.
//!
//! In:  nothing
//! Out: every register preserved (it uses none)
//! Nothing if `g_patchesApplied` (0x71C72C) is not 0; else that becomes 1 and:
//! - twelve settings are held to their ranges (`patches::RANGES`; the comparisons look at
//!   the sign of the difference only, which is kept);
//! - `flt_613FC8` = 9.8; twelve records of 0x30 bytes from 0x6139AC on get 1.0, 0.9 and
//!   1.7 (1.2 in the fourth and the eleventh) in their first three floats; `dword_613AE8`
//!   and `dword_613B18` = 10.0;
//! - if the setting at 0x71C6E0 is above 100, `flt_71C708` = it / 100;
//! - only if the settings' switch `dword_71C6D0` is not 0: the ten floats from
//!   `flt_71C704` on = the ten settings from 0x71C6DC on, each / 100; `flt_613FC8` times
//!   the first; every record's three floats times one of six of them, by the record
//!   (`patches::BY_RECORD`), and times the fifth, the fourth and the third; and with the setting at
//!   0x71C6D8 at 0, `dword_613AE8` and `dword_613B18` = 0.
//!
//! The settings' keys in `var\ral.cfg` say what they are, from 0x71C6D0 on: `fizpatch`
//! (the switch), `patchmaxdrawdistance`, `patchstickygrass`, `patchgravity`,
//! `patchdamages`, `patchbrakegrip`, `patchaccelgrip`, `patchslidegrip`, `patchgripgrass`,
//! `patchgriptarmac`, `patchgripsnow`, `patchgripmud`, `patchgripgravel`, the last ten in
//! percent. So `flt_613FC8` is gravity, and a record is a kind of ground with its slide,
//! acceleration and brake grip: by `BY_RECORD` tarmac, gravel, mud, snow, grass, mud,
//! grass, grass, snow, grass, snow, gravel. A reading of the names, not seen in a race.
//! Callers: `system::load_game_data`, and two that copy thirteen settings in from
//! `unk_5C2C98`, clear the switch and call it again. The arithmetic is plain floats
//! (`rc2k_core::physics::patches`): each figure is one correctly rounded quotient or
//! product at the game's double and extended precision.

use crate::physics::tables_store::Surface;
use crate::race::owner::Race;
use crate::race::owner_shared::Shared;
use crate::session::Float;
use rc2k_core::physics::patches::{Patches, apply_patches as apply_patches_core};

/// The twelve ground records patched: `PHYSICS.surfaces[1 ..= 12]`.
fn records(shared: &Shared) -> &[Surface] {
    &shared.physics_tables.surfaces[1..13]
}

/// A record's three grips: slide, acceleration, brake.
fn grips(r: &Surface) -> [&Float; 3] {
    [&r.slide_grip, &r.accel_grip, &r.brake_grip]
}

/// The two drags (10.0) the sticky grass setting keeps: grass, records 7 and 8.
fn tens(shared: &Shared) -> [&Float; 2] {
    [
        &shared.physics_tables.surfaces[7].drag,
        &shared.physics_tables.surfaces[8].drag,
    ]
}

/// The game's patch state.
fn load(shared: &Shared) -> Patches {
    let w = &shared.work;
    Patches {
        switch: w.patch_switch.get(),
        settings: std::array::from_fn(|n| w.patch_settings[n].get()),
        factors: std::array::from_fn(|n| w.patch_factors[n].bits()),
        applied: w.patches_applied.get(),
        gravity: shared.physics_tables.gravity.bits(),
        records: std::array::from_fn(|r| grips(&records(shared)[r]).map(Float::bits)),
        tens: tens(shared).map(Float::bits),
    }
}

/// The places that changed written back (the original writes only these, or the same
/// value again).
fn store(shared: &Shared, before: &Patches, after: &Patches) {
    let w = &shared.work;
    if before.applied != after.applied {
        w.patches_applied.set(after.applied);
    }
    for n in 0..12 {
        if before.settings[n] != after.settings[n] {
            w.patch_settings[n].set(after.settings[n]);
        }
    }
    let put = |f: &Float, old: u32, new: u32| {
        if old != new {
            f.set_bits(new);
        }
    };
    put(
        &shared.physics_tables.gravity,
        before.gravity,
        after.gravity,
    );
    for (r, record) in records(shared).iter().enumerate() {
        for (n, f) in grips(record).into_iter().enumerate() {
            put(f, before.records[r][n], after.records[r][n]);
        }
    }
    for (k, f) in tens(shared).into_iter().enumerate() {
        put(f, before.tens[k], after.tens[k]);
    }
    for n in 0..10 {
        put(&w.patch_factors[n], before.factors[n], after.factors[n]);
    }
}

/// The Rust version (the original's to the bit at the game's double and extended
/// precision: each figure one quotient, or a product whose first step is exact).
// Converted: `rc2k_core::physics::patches::apply_patches`.
pub fn apply_patches(race: &Race) {
    let before = load(&race.shared);
    if before.applied != 0 {
        return;
    }
    let mut after = before;
    apply_patches_core(&mut after, race.course.numbers.hundred.get());
    store(&race.shared, &before, &after);
}
