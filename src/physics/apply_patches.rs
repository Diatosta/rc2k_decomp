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
//! `unk_5C2C98`, clear the switch and call it again.
//! The arithmetic is plain floats (`rc2k_core::physics::patches`): each figure is one
//! correctly rounded quotient or product at the game's double and extended precision.
//! Self-test: settings of the test's in the game's places, at those two precisions.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_read, game_write, rva};
use crate::screens::harness::Left;
use crate::service::harness::state_entry;
use rc2k_core::physics::patches::{Patches, apply_patches as apply_patches_core};

// ---- game data ----

/// dword_71C6D0: the settings' switch, then twelve settings (0x71C6D4 .. 0x71C700), ten
/// floats (0x71C704 .. 0x71C728) and `g_patchesApplied` (0x71C72C).
const SETTINGS_RVA: usize = rva(0x71C6D0);
const FACTORS_RVA: usize = rva(0x71C704);
const APPLIED_RVA: usize = rva(0x71C72C);
const HUNDRED_RVA: usize = rva(0x5C5D14); // flt_5C5D14, 100.0
const GRAVITY_RVA: usize = rva(0x613FC8); // flt_613FC8
const RECORDS_RVA: usize = rva(0x6139AC);
const RECORD_LEN: usize = 0x30;
const RECORDS: usize = 12;
const TENS_RVA: [usize; 2] = [rva(0x613AE8), rva(0x613B18)];

// ---- Rust version ----

fn record_float(record: usize, n: usize) -> usize {
    RECORDS_RVA + record * RECORD_LEN + n * 4
}

/// The game's patch state.
unsafe fn load() -> Patches {
    unsafe {
        let d = |a: usize| game_read::<u32>(a);
        Patches {
            switch: d(SETTINGS_RVA),
            settings: std::array::from_fn(|n| d(SETTINGS_RVA + 4 + n * 4)),
            factors: std::array::from_fn(|n| d(FACTORS_RVA + n * 4)),
            applied: d(APPLIED_RVA),
            gravity: d(GRAVITY_RVA),
            records: std::array::from_fn(|r| std::array::from_fn(|n| d(record_float(r, n)))),
            tens: TENS_RVA.map(d),
        }
    }
}

/// The places that changed written back (the original writes only these, or the same
/// value again).
unsafe fn store(before: &Patches, after: &Patches) {
    let put = |a: usize, old: u32, new: u32| {
        if old != new {
            unsafe { game_write(a, new) };
        }
    };
    put(APPLIED_RVA, before.applied, after.applied);
    for n in 0..12 {
        put(
            SETTINGS_RVA + 4 + n * 4,
            before.settings[n],
            after.settings[n],
        );
    }
    put(GRAVITY_RVA, before.gravity, after.gravity);
    for r in 0..RECORDS {
        for n in 0..3 {
            put(
                record_float(r, n),
                before.records[r][n],
                after.records[r][n],
            );
        }
    }
    for (k, a) in TENS_RVA.into_iter().enumerate() {
        put(a, before.tens[k], after.tens[k]);
    }
    for n in 0..10 {
        put(FACTORS_RVA + n * 4, before.factors[n], after.factors[n]);
    }
}

/// The Rust version (the original's to the bit at the game's double and extended
/// precision: each figure one quotient, or a product whose first step is exact).
// Converted: `rc2k_core::physics::patches::apply_patches`.
pub unsafe fn apply_patches() {
    unsafe {
        let before = load();
        if before.applied != 0 {
            return;
        }
        let mut after = before;
        apply_patches_core(&mut after, game_read(HUNDRED_RVA));
        store(&before, &after);
    }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { apply_patches() };
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
