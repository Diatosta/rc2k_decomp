//! `physics::apply_patches` (0x4648BD): the physics' constants, with the `patch` settings
//! worked into them. Once.
//!
//! In:  nothing
//! Out: every register preserved (it uses none)
//! Nothing if `g_patchesApplied` (0x71C72C) is not 0; else that becomes 1 and:
//! - twelve settings are held to their ranges (`RANGES`; the comparisons look at the sign
//!   of the difference only, which is kept);
//! - `flt_613FC8` = 9.8; twelve records of 0x30 bytes from 0x6139AC on get 1.0, 0.9 and
//!   1.7 (1.2 in the fourth and the eleventh) in their first three floats; `dword_613AE8`
//!   and `dword_613B18` = 10.0;
//! - if the setting at 0x71C6E0 is above 100, `flt_71C708` = it / 100;
//! - only if the settings' switch `dword_71C6D0` is not 0: the ten floats from
//!   `flt_71C704` on = the ten settings from 0x71C6DC on, each / 100; `flt_613FC8` times
//!   the first; every record's three floats times one of six of them, by the record
//!   (`BY_RECORD`), and times the fifth, the fourth and the third; and with the setting at
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
//! The arithmetic is the x87's, done here with the same instructions, so that the
//! rounding is that of the control word in force, as in the original.
//! Self-test: settings of the test's in the game's places, under three precisions.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("physics::apply_patches", rva(0x4648BD), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

/// dword_71C6D0: the settings' switch, then twelve settings (0x71C6D4 .. 0x71C700), ten
/// floats (0x71C704 .. 0x71C728) and `g_patchesApplied` (0x71C72C).
const SETTINGS_RVA: usize = rva(0x71C6D0);
const SETTINGS_LEN: usize = 0x60;
const FACTORS_RVA: usize = rva(0x71C704);
const APPLIED_RVA: usize = rva(0x71C72C);
const HUNDRED_RVA: usize = rva(0x5C5D14); // flt_5C5D14, 100.0
const GRAVITY_RVA: usize = rva(0x613FC8); // flt_613FC8
const RECORDS_RVA: usize = rva(0x6139AC);
const RECORD_LEN: usize = 0x30;
const RECORDS: usize = 12;
const TENS_RVA: [usize; 2] = [rva(0x613AE8), rva(0x613B18)];
/// The settings from 0x71C6D4 on: the least and the most each may be.
const RANGES: [(u32, u32); 12] = [
    (4, 0x10),
    (0, 1),
    (10, 200),
    (0, 100_000),
    (10, 200),
    (10, 200),
    (10, 200),
    (10, 200),
    (10, 200),
    (10, 200),
    (10, 200),
    (10, 200),
];
/// A record's three floats to begin with, as bits: 1.0, 0.9 (one bit above the nearest
/// float) and 1.7.
const PLAIN: [u32; 3] = [0x3F80_0000, 0x3F66_6667, 0x3FD9_999A];
/// The third float of these records: 1.2.
const OTHER_THIRD: ([usize; 2], u32) = ([3, 10], 0x3F99_999A);
const NINE_POINT_EIGHT: u32 = 0x411C_CCCD;
const TEN: u32 = 0x4120_0000;
/// By record: which of the ten factors its three floats are multiplied by first.
const BY_RECORD: [usize; RECORDS] = [6, 9, 8, 7, 5, 8, 5, 5, 7, 5, 7, 9];
/// The factors the first, second and third float are then multiplied by.
const BY_FLOAT: [usize; 3] = [4, 3, 2];

// ---- the x87's arithmetic ----

macro_rules! x87 {
    ($($line:literal),+; $($name:ident = $value:expr),+) => {
        unsafe {
            core::arch::asm!(
                $($line),+,
                $($name = in(reg) $value),+,
                out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
                out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
                options(nostack),
            )
        }
    };
}

/// `fild number; fdiv by; fstp`.
fn divided(number: i32, by: f32) -> f32 {
    let mut out = 0f32;
    x87!("fild dword ptr [{n}]", "fdiv dword ptr [{b}]", "fstp dword ptr [{o}]";
        n = &raw const number, b = &raw const by, o = &raw mut out);
    out
}

/// `fld a; fmul b; fstp`.
fn times(a: f32, b: f32) -> f32 {
    let mut out = 0f32;
    x87!("fld dword ptr [{a}]", "fmul dword ptr [{b}]", "fstp dword ptr [{o}]";
        a = &raw const a, b = &raw const b, o = &raw mut out);
    out
}

/// `fld a; fmul b; fmul c; fstp`.
fn times_both(a: f32, b: f32, c: f32) -> f32 {
    let mut out = 0f32;
    x87!("fld dword ptr [{a}]", "fmul dword ptr [{b}]", "fmul dword ptr [{c}]",
        "fstp dword ptr [{o}]";
        a = &raw const a, b = &raw const b, c = &raw const c, o = &raw mut out);
    out
}

// ---- Rust version ----

pub unsafe fn apply_patches() {
    unsafe {
        if game_read::<u32>(APPLIED_RVA) != 0 {
            return;
        }
        game_write(APPLIED_RVA, 1u32);
        let setting = |n: usize| SETTINGS_RVA + 4 + n * 4;
        for (n, (least, most)) in RANGES.into_iter().enumerate() {
            // `cmp; jns` and `cmp; js`: the sign of the difference, whatever the overflow.
            if (game_read::<u32>(setting(n)).wrapping_sub(least) as i32) < 0 {
                game_write(setting(n), least);
            }
            if (game_read::<u32>(setting(n)).wrapping_sub(most) as i32) >= 0 {
                game_write(setting(n), most);
            }
        }
        game_write(GRAVITY_RVA, NINE_POINT_EIGHT);
        let float_of = |record: usize, n: usize| RECORDS_RVA + record * RECORD_LEN + n * 4;
        for record in 0..RECORDS {
            for (n, value) in PLAIN.into_iter().enumerate() {
                let value = if n == 2 && OTHER_THIRD.0.contains(&record) {
                    OTHER_THIRD.1
                } else {
                    value
                };
                game_write(float_of(record, n), value);
            }
        }
        for ten in TENS_RVA {
            game_write(ten, TEN);
        }
        let hundred: f32 = game_read(HUNDRED_RVA);
        if game_read::<u32>(setting(3)) > 100 {
            game_write(
                FACTORS_RVA + 4,
                divided(game_read::<i32>(setting(3)), hundred),
            );
        }
        if game_read::<u32>(SETTINGS_RVA) == 0 {
            return;
        }
        for n in 0..10 {
            game_write(
                FACTORS_RVA + n * 4,
                divided(game_read::<i32>(setting(2 + n)), hundred),
            );
        }
        let factor = |n: usize| game_read::<f32>(FACTORS_RVA + n * 4);
        game_write(GRAVITY_RVA, times(game_read(GRAVITY_RVA), factor(0)));
        for (record, first) in BY_RECORD.into_iter().enumerate() {
            for (n, then) in BY_FLOAT.into_iter().enumerate() {
                let at = float_of(record, n);
                game_write(at, times_both(game_read(at), factor(first), factor(then)));
            }
        }
        if game_read::<u32>(setting(1)) == 0 {
            for ten in TENS_RVA {
                game_write(ten, 0u32);
            }
        }
    }
}

// ---- hook ----

fn regions() -> Vec<Region> {
    let mut regions = vec![
        (game_addr(SETTINGS_RVA) as u32, SETTINGS_LEN),
        (game_addr(GRAVITY_RVA) as u32, 4),
    ];
    regions.extend(
        (0..RECORDS).map(|record| (game_addr(RECORDS_RVA + record * RECORD_LEN) as u32, 12)),
    );
    regions.extend(TENS_RVA.map(|ten| (game_addr(ten) as u32, 4)));
    regions
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { apply_patches() };
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

// ---- self-test ----

/// Runs `f` with the x87's control word set to `word`.
fn with_control_word(word: u16, f: &mut dyn FnMut()) {
    let mut old = 0u16;
    unsafe {
        core::arch::asm!("fnstcw word ptr [{o}]", "fldcw word ptr [{w}]",
            o = in(reg) &raw mut old, w = in(reg) &raw const word, options(nostack));
    }
    f();
    unsafe {
        core::arch::asm!("fldcw word ptr [{w}]", w = in(reg) &raw const old, options(nostack));
    }
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if unsafe { game_read::<u32>(HUNDRED_RVA) } != 100f32.to_bits() {
        return t.skip("the divisor is not 100.0");
    }
    let regions = regions();
    let saved = capture(&regions);
    // (what, the switch, the twelve settings, already applied)
    let m = u32::MAX;
    let cases: [(&str, u32, [u32; 12], u32); 10] = [
        (
            "all 100, switched off",
            0,
            [8, 1, 100, 100, 100, 100, 100, 100, 100, 100, 100, 100],
            0,
        ),
        (
            "all 100, switched on",
            1,
            [8, 1, 100, 100, 100, 100, 100, 100, 100, 100, 100, 100],
            0,
        ),
        (
            "mixed, on",
            1,
            [5, 0, 37, 4321, 11, 199, 73, 150, 99, 101, 33, 177],
            0,
        ),
        (
            "mixed, off",
            0,
            [5, 0, 37, 4321, 11, 199, 73, 150, 99, 101, 33, 177],
            0,
        ),
        (
            "thirds, on",
            7,
            [16, 1, 133, 101, 67, 33, 167, 30, 70, 110, 130, 190],
            0,
        ),
        (
            "below the ranges",
            1,
            [0, m, 9, m - 5, 0, 1, 2, 3, 4, 5, 6, m],
            0,
        ),
        (
            "above the ranges",
            1,
            [
                17,
                2,
                201,
                100_001,
                300,
                999,
                201,
                5000,
                256,
                1 << 20,
                202,
                0x7FFF_FFFF,
            ],
            0,
        ),
        (
            "far outside",
            1,
            [
                1 << 31,
                (1 << 31) + 1,
                (1 << 31) + 10,
                (1 << 31) - 1,
                0x8000_00C8,
                0x8000_000A,
                0x7FFF_FFF0,
                0xC000_0000,
                0x4000_0000,
                0x8000_0009,
                0xFFFF_FF00,
                0x8000_00C9,
            ],
            0,
        ),
        (
            "applied before",
            1,
            [5, 0, 37, 4321, 11, 199, 73, 150, 99, 101, 33, 177],
            1,
        ),
        (
            "applied before, another mark",
            0,
            [300, 9, 37, 4321, 11, 199, 73, 150, 99, 101, 33, 177],
            2,
        ),
    ];
    // Precision of 53 bits (what a process starts with), 64 and 24.
    for word in [0x027Fu16, 0x037F, 0x007F] {
        for (what, switch, settings, applied) in cases {
            with_control_word(word, &mut || {
                check_state(
                    &mut t,
                    &format!("control word {word:#x}, {what}"),
                    call_original,
                    Regs::SENTINEL,
                    &|| unsafe {
                        restore(&regions, &vec![0xAA; saved.len()]);
                        game_write(SETTINGS_RVA, switch);
                        for (n, value) in settings.into_iter().enumerate() {
                            game_write(SETTINGS_RVA + 4 + n * 4, value);
                        }
                        for n in 0..10 {
                            game_write(FACTORS_RVA + n * 4, 1.0f32 + n as f32 / 7.0);
                        }
                        game_write(APPLIED_RVA, applied);
                    },
                    &regions,
                    &rust,
                );
            });
        }
    }
    restore(&regions, &saved);
    t.finish()
}
