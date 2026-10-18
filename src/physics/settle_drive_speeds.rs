//! `physics::settle_drive_speeds` (0x43BF6F): driven speeds brought toward 0 by a step
//! (`flt_613F3C`, 1.0, when `dword_71A614` is set, else `flt_613F40`, 0.01; stored in
//! `dword_62400C`), never past it: `g_carDriveSpeed` when `dword_71A220` is set and the car
//! is in neutral, `g_carDriveSpeed2` when `dword_71A224` is set (whatever the gear). A
//! speed of 0 (or NaN) is left; one that reaches or crosses 0 becomes +0 (a NaN after the
//! step too). Then, in gear, `physics::revs_from_wheels`.
//!
//! In:  nothing
//! Out: in gear, ebx = `g_carGear` (`revs_from_wheels`); the rest kept; no flags read
//!
//! Caller: `sub_43B635` (0x43B7AD, before `sub_43EA54`), every physics step.
//! Refuses (before anything) a gear outside -1 ..= 7 (`revs_from_wheels`' table).
//! Self-test: a copy of the original (its variables the test's, its call to a copy of
//! `revs_from_wheels` on the same) against the Rust core: gears, the three switches,
//! speeds by lot and near the step, under three precisions.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, game_read, hook_asm, rva};
use crate::math::x87;
use crate::math::x87_run::{TEST_CONTROL_WORDS, with_control_word};
use crate::physics::gearing::{GearPlaces, TEST_GEARS, TestGearing, revs_from_wheels_at};
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, dispatch_state};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "physics::settle_drive_speeds",
    rva(0x43BF6F),
    thunk,
    &ORIGINAL,
)
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const FAST_STEP_RVA: usize = rva(0x613F3C); // flt_613F3C, 1.0
const SLOW_STEP_RVA: usize = rva(0x613F40); // flt_613F40, 0.01

/// The switches and the scratch: the game's or a test's.
#[derive(Clone, Copy)]
pub struct SettlePlaces {
    /// `dword_71A220`: settle the first driven speed (in neutral).
    pub first: u32,
    /// `dword_71A224`: settle the second.
    pub second: u32,
    /// `dword_71A614`: the big step.
    pub fast: u32,
    /// `dword_62400C`.
    pub scratch: u32,
}

impl SettlePlaces {
    pub fn game() -> Self {
        let g = |a: usize| game_addr(rva(a)) as u32;
        Self {
            first: g(0x71A220),
            second: g(0x71A224),
            fast: g(0x71A614),
            scratch: g(0x62400C),
        }
    }
}

unsafe fn at(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

unsafe fn put(a: u32, v: u32) {
    unsafe { (a as *mut u32).write_unaligned(v) }
}

/// One speed at `v` toward 0 by the step at `step`.
unsafe fn settle(v: u32, step: u32) {
    unsafe {
        let x = f32::from_bits(at(v));
        if x == 0.0 || x.is_nan() {
            return;
        }
        if x < 0.0 {
            let y = x87::add(at(v), at(step));
            put(v, y);
            if f32::from_bits(y) >= 0.0 {
                put(v, 0);
            }
        } else {
            let y = x87::sub(at(v), at(step));
            put(v, y);
            let y = f32::from_bits(y);
            if y < 0.0 || y.is_nan() {
                put(v, 0);
            }
        }
    }
}

/// The Rust core.
pub unsafe fn settle_drive_speeds_at(g: &GearPlaces, s: &SettlePlaces) -> Result<Left, Refused> {
    unsafe {
        let gear = at(g.gear);
        if gear != 0 && !(-1..=7).contains(&(gear as i32)) {
            return Err(BadInput("a gear outside g_gearRatios (-1 ..= 7)").into());
        }
        let step = || {
            game_read::<u32>(if at(s.fast) != 0 {
                FAST_STEP_RVA
            } else {
                SLOW_STEP_RVA
            })
        };
        if at(s.first) != 0 && gear == 0 {
            put(s.scratch, step());
            settle(g.drive_speed, s.scratch);
        }
        if at(s.second) != 0 {
            put(s.scratch, step());
            settle(g.drive_speed_2, s.scratch);
        }
        if gear != 0 {
            return revs_from_wheels_at(g);
        }
        Ok(Left::nothing())
    }
}

/// The Rust version.
pub unsafe fn settle_drive_speeds() -> Result<Left, Refused> {
    unsafe { settle_drive_speeds_at(&GearPlaces::game(), &SettlePlaces::game()) }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { settle_drive_speeds() }
}

extern "C" fn dispatch(r: &mut Regs) {
    let mut regions: Vec<Region> = GearPlaces::game().regions();
    regions.push((SettlePlaces::game().scratch, 4));
    dispatch_state(&HOOK, r, call_original, &regions, &rust);
}

/// Speeds the test puts in now and then: about the slow step, about 0, NaN.
const EDGE_SPEEDS: [u32; 8] = [
    0x3C23_D70A, // 0.01
    0xBC23_D70A, // -0.01
    0x3BA3_D70A, // 0.005
    0xBF00_0000, // -0.5
    0x0000_0000,
    0x8000_0000,
    0x7FC0_0000,
    0x3F80_0001, // just over 1
];

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let test = TestGearing::new();
    let mut cells = vec![0u32; 4];
    let base = cells.as_mut_ptr() as u32;
    let s = SettlePlaces {
        first: base,
        second: base + 4,
        fast: base + 8,
        scratch: base + 12,
    };
    let revs = match crate::audio::test_copy::make(
        &crate::physics::revs_from_wheels::HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x43C0C9,
            len: 0x38,
            entry: &[0x83, 0x3D, 0xAC, 0xA1, 0x71, 0x00, 0x00],
            calls: &[],
            jumps_out: &[],
        },
        &|_| None,
        &test.variables(),
        7,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    let target = revs.addr();
    let p = test.places;
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x43BF6F,
            len: 0x15A,
            entry: &[0x83, 0x3D, 0x20, 0xA2, 0x71, 0x00, 0x00],
            calls: &[0x154],
            jumps_out: &[],
        },
        &|r| (r == rva(0x43C0C9)).then_some(target),
        &[
            (0x71A220, s.first),
            (0x71A1AC, p.gear),
            (0x71A614, s.fast),
            (0x62400C, s.scratch),
            (0x71A190, p.drive_speed),
            (0x71A224, s.second),
            (0x71A194, p.drive_speed_2),
        ],
        30,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    let mut regions = p.regions();
    regions.push((p.gear, 4));
    regions.push((base, 16));
    for word in TEST_CONTROL_WORDS {
        with_control_word(word, &mut || {
            for gear in TEST_GEARS {
                for switches in 0..8u32 {
                    for seed in 1..=3u32 {
                        let seed = seed + gear.wrapping_mul(7) + switches * 31;
                        let fill = || unsafe {
                            test.fill(gear, seed % 3, seed);
                            if seed % 2 == 0 {
                                put(p.drive_speed, EDGE_SPEEDS[(seed / 2 % 8) as usize]);
                                put(p.drive_speed_2, EDGE_SPEEDS[(seed / 3 % 8) as usize]);
                            }
                            put(s.first, switches & 1);
                            put(s.second, (switches >> 1) & 1);
                            put(s.fast, (switches >> 2) & 1);
                            put(s.scratch, 0xAAAA_AAAA);
                        };
                        fill();
                        let mut ro = Regs::SENTINEL;
                        unsafe { call_game(&mut ro, copy.addr()) };
                        let theirs = capture(&regions);
                        fill();
                        let mine = unsafe { settle_drive_speeds_at(&p, &s) };
                        let state = capture(&regions);
                        let regs_ok = mine.as_ref().is_ok_and(|l| l.matches(&Regs::SENTINEL, &ro));
                        t.check(regs_ok && theirs == state, || {
                            format!(
                                "control word {word:#x}, gear {gear:#x}, switches {switches}, seed {seed}: \
                                 regs_ok={regs_ok} original {theirs:x?}; rust {state:x?}"
                            )
                        });
                    }
                }
            }
        });
    }
    unsafe { put(p.gear, 9) };
    let refused = unsafe { settle_drive_speeds_at(&p, &s) }.is_err();
    t.check(refused, || "gear 9 not refused".into());
    drop((copy, revs, test, cells));
    t.finish()
}
