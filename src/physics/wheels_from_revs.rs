//! `physics::wheels_from_revs` (0x43C124): in gear, `g_carDriveSpeed` = `g_carEngineSpeed`
//! times `g_gearRatios[g_carGear]`; with `g_carDriveKind` 1 `g_carDriveSpeed2` the same
//! (`fld; fstp` of it). Nothing in neutral.
//!
//! In:  nothing (the current car's work copy, `drive/car.rs`)
//! Out: ebx = `g_carGear` when not 0 (else kept); the rest kept
//!
//! See `physics/gearing.rs`. Callers (8): the gearbox under `sub_4454D2` (`sub_43B635`,
//! `sub_43C15C`, `sub_43C20D`, `sub_43C2AE`, `sub_43C5C2`, `sub_43C8AC`, `sub_43EA63`, and
//! unowned code at 0x43CA7B).
//! Refuses a gear outside -1 ..= 7 (the ratio table), before writing.
//! Self-test: a copy of the original whose variables are the test's (they are the car's,
//! used all through a frame), against the converted version on the same: gears, drive kinds and
//! speeds by lot, exact at double precision (one correctly rounded
//! product or quotient; at extended a sum of far-apart speeds may round otherwise);
//! and the refusal.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, call_game, hook_asm, rva};
use crate::math::x87_run::{TEST_CONTROL_WORDS, with_control_word};
use crate::physics::gearing::*;
use crate::screens::harness::Left;
use crate::service::harness::capture;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("physics::wheels_from_revs", rva(0x43C124), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The Rust version (the exact one, for the gearbox's Rust callers), on the game's
/// variables.
pub unsafe fn wheels_from_revs() -> Result<Left, Refused> {
    unsafe { wheels_from_revs_at(&GearPlaces::game()) }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { wheels_from_revs_plain_at(&GearPlaces::game()) }
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_gearing(&HOOK, r, call_original, &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let test = TestGearing::new();
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x43C124,
            len: 0x38,
            entry: &[0x83, 0x3D, 0xAC, 0xA1, 0x71, 0x00, 0x00],
            calls: &[],
            jumps_out: &[],
        },
        &|_| None,
        &test.variables(),
        8,
    ) {
        Ok(copy) => copy,
        Err(why) => return t.skip(&why),
    };
    let p = test.places;
    let mut regions = p.regions();
    regions.push((p.gear, 4));
    for word in TEST_CONTROL_WORDS {
        with_control_word(word, &mut || {
            for gear in TEST_GEARS {
                for kind in TEST_KINDS {
                    for seed in 1..=3u32 {
                        let seed = seed + gear.wrapping_mul(7) + kind * 31;
                        test.fill(gear, kind, seed);
                        let mut ro = Regs::SENTINEL;
                        unsafe { call_game(&mut ro, copy.addr()) };
                        let theirs = capture(&regions);
                        test.fill(gear, kind, seed);
                        let mine = unsafe { wheels_from_revs_plain_at(&p) };
                        let state = capture(&regions);
                        let regs_ok = mine.as_ref().is_ok_and(|l| l.matches(&Regs::SENTINEL, &ro));
                        t.check(regs_ok && speeds_agree(&regions, &theirs, &state, word), || {
                            format!(
                                "control word {word:#x}, gear {gear:#x}, kind {kind}, seed {seed}: \
                                 regs_ok={regs_ok} original {theirs:x?}; rust {mine:?} {state:x?}"
                            )
                        });
                    }
                }
            }
        });
    }
    for gear in [8u32, 0xFFFF_FFFE, 0x8000_0000] {
        test.fill(gear, 1, 5);
        let before = capture(&regions);
        let refused = unsafe { wheels_from_revs_plain_at(&p) }.is_err();
        t.check(refused && capture(&regions) == before, || {
            format!("gear {gear:#x}: refused {refused}, or something written")
        });
    }
    t.finish()
}
