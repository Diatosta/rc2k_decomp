//! `physics::cap_drive_speed` (0x440DF3): a slip allowance `flt_614168` =
//! `flt_613E48` / (|`g_carSpeed`| + `flt_613E4C`) - `flt_613E50`, held at 0 or more (an
//! unordered one counts as below); `g_carDriveSpeed` held to at most the speed plus it
//! (through `dword_62400C`; a drive speed below that, or unordered, is left).
//!
//! In:  nothing
//! Out: every register kept; no flags read (the caller's next is a `mov`)
//!
//! Caller: `sub_43DFB1` (0x43E0DD).
//! Self-test: a copy of the original (its four variables the test's) against the core:
//! speeds and drive speeds by lot and about the cap, under three precisions.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, game_read, hook_asm, rva};
use crate::math::x87;
use crate::math::x87_run::{TEST_CONTROL_WORDS, over_abs_plus_minus, with_control_word};
use crate::scene::matrix3::test_floats;
use crate::screens::harness::Left;
use crate::service::harness::{capture, dispatch_state};
use std::cmp::Ordering;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("physics::cap_drive_speed", rva(0x440DF3), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// C0 of comparing `a` with `b`: below, or unordered.
fn below(a: u32, b: u32) -> bool {
    !matches!(
        f32::from_bits(a).partial_cmp(&f32::from_bits(b)),
        Some(Ordering::Greater | Ordering::Equal)
    )
}

unsafe fn at(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

unsafe fn put(a: u32, v: u32) {
    unsafe { (a as *mut u32).write_unaligned(v) }
}

fn k(a: usize) -> u32 {
    unsafe { game_read(rva(a)) }
}

/// The places: the allowance, the speed, the scratch, the drive speed.
pub type CapPlaces = [u32; 4];

/// The core.
pub unsafe fn cap_drive_speed_at(v: CapPlaces) {
    unsafe {
        let zero = k(0x5C5D90);
        put(
            v[0],
            over_abs_plus_minus(k(0x613E48), at(v[1]), k(0x613E4C), k(0x613E50)),
        );
        if below(at(v[0]), zero) {
            put(v[0], x87::loaded(zero));
        }
        put(v[2], x87::add(at(v[0]), at(v[1])));
        if !below(at(v[3]), at(v[2])) {
            put(v[3], x87::loaded(at(v[2])));
        }
    }
}

fn game() -> CapPlaces {
    let g = |a: usize| game_addr(rva(a)) as u32;
    [g(0x614168), g(0x71A184), g(0x62400C), g(0x71A190)]
}

/// The Rust version.
pub unsafe fn cap_drive_speed() {
    unsafe { cap_drive_speed_at(game()) }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { cap_drive_speed() };
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    let v = game();
    dispatch_state(
        &HOOK,
        r,
        call_original,
        &[(v[0], 4), (v[2], 4), (v[3], 4)],
        &rust,
    );
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut cells = vec![0u32; 4];
    let base = cells.as_mut_ptr() as u32;
    let v: CapPlaces = [base, base + 4, base + 8, base + 12];
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x440DF3,
            len: 0x79,
            entry: &[0xD9, 0x05, 0x48, 0x3E, 0x61, 0x00],
            calls: &[],
            jumps_out: &[],
        },
        &|_| None,
        &[
            (0x614168, v[0]),
            (0x71A184, v[1]),
            (0x62400C, v[2]),
            (0x71A190, v[3]),
        ],
        11,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    let all = [(base, 16)];
    for word in TEST_CONTROL_WORDS {
        with_control_word(word, &mut || {
            for seed in 0..12u32 {
                let fill = || {
                    let mut f = [0u32; 2];
                    test_floats(seed * 11 + 4, &mut f);
                    let s = (f32::from_bits(f[0]) % 60.0).to_bits();
                    let d = match seed {
                        0 => 0x7FC0_0000,
                        1 => 1.0e9f32.to_bits(),
                        2 => s,
                        _ => (f32::from_bits(f[1]) % 80.0).to_bits(),
                    };
                    unsafe { (base as *mut [u32; 4]).write([0xAAAA_AAAA, s, 0xAAAA_AAAA, d]) };
                };
                fill();
                let mut ro = Regs::SENTINEL;
                unsafe { call_game(&mut ro, copy.addr()) };
                let theirs = capture(&all);
                fill();
                unsafe { cap_drive_speed_at(v) };
                let mine = capture(&all);
                let regs_ok = ro.preserved_except(&Regs::SENTINEL, 0);
                t.check(regs_ok && theirs == mine, || {
                    format!("control word {word:#x}, seed {seed}: regs_ok={regs_ok} original {theirs:x?}; rust {mine:x?}")
                });
            }
        });
    }
    drop((copy, cells));
    t.finish()
}
