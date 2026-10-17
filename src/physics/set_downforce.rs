//! `physics::set_downforce` (0x43F508): unless `dword_71A224` is set: `flt_614174` =
//! (`g_carSpeed`² `flt_613FB4` + `dword_61476C`) `flt_613FC8` (gravity, by
//! `physics::apply_patches`), `flt_6147F4` its negative, `flt_614804` = -1. By the look
//! of it the car's weight with downforce growing with the speed's square (a reading).
//!
//! In:  nothing
//! Out: every register kept; no flags read
//!
//! Caller: `sub_43B635` (0x43B728), every physics step.
//! Self-test: a copy of the original on test variables (the two constants the game's),
//! speeds and offsets by lot, the switch on and off, under three precisions.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, game_read, hook_asm, rva};
use crate::math::x87_run::{
    TEST_CONTROL_WORDS, minus_one, square_times_plus_times, with_control_word,
};
use crate::scene::matrix3::test_floats;
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, dispatch_state};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("physics::set_downforce", rva(0x43F508), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The variables (IDA addresses): the switch, the speed, the offset, and the three
/// written.
const VARIABLES: [usize; 6] = [0x71A224, 0x71A184, 0x61476C, 0x614174, 0x6147F4, 0x614804];
const FACTOR_RVA: usize = rva(0x613FB4); // flt_613FB4
const GRAVITY_RVA: usize = rva(0x613FC8); // flt_613FC8

unsafe fn at(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

unsafe fn put(a: u32, v: u32) {
    unsafe { (a as *mut u32).write_unaligned(v) }
}

/// The Rust core on the variables at `v`.
pub unsafe fn set_downforce_at(v: &[u32; 6]) {
    unsafe {
        if at(v[0]) != 0 {
            return;
        }
        let (load, negative) = square_times_plus_times(
            at(v[1]),
            game_read(FACTOR_RVA),
            at(v[2]),
            game_read(GRAVITY_RVA),
        );
        put(v[3], load);
        put(v[4], negative);
        put(v[5], minus_one());
    }
}

fn game() -> [u32; 6] {
    VARIABLES.map(|x| game_addr(rva(x)) as u32)
}

/// The Rust version.
pub unsafe fn set_downforce() {
    unsafe { set_downforce_at(&game()) }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { set_downforce() };
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    let v = game();
    dispatch_state(
        &HOOK,
        r,
        call_original,
        &[(v[3], 4), (v[4], 4), (v[5], 4)],
        &rust,
    );
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut cells = vec![0u32; 6];
    let base = cells.as_mut_ptr() as u32;
    let v: [u32; 6] = std::array::from_fn(|k| base + 4 * k as u32);
    let variables: Vec<(usize, u32)> = VARIABLES.iter().zip(v).map(|(&g, t)| (g, t)).collect();
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x43F508,
            len: 0x3C,
            entry: &[0x83, 0x3D, 0x24, 0xA2, 0x71, 0x00, 0x00],
            calls: &[],
            jumps_out: &[],
        },
        &|_| None,
        &variables,
        6,
    ) {
        Ok(copy) => copy,
        Err(why) => return t.skip(&why),
    };
    let all: [Region; 1] = [(base, 24)];
    for word in TEST_CONTROL_WORDS {
        with_control_word(word, &mut || {
            for seed in 1..=10u32 {
                for switch in [0u32, 1] {
                    let fill = || {
                        let mut f = [0u32; 2];
                        test_floats(seed * 7 + 1, &mut f);
                        let c = [switch, f[0], f[1], 0xAAAA_AAAA, 0xAAAA_AAAA, 0xAAAA_AAAA];
                        for (k, x) in c.into_iter().enumerate() {
                            unsafe { (v[k] as *mut u32).write_volatile(x) };
                        }
                    };
                    fill();
                    let mut ro = Regs::SENTINEL;
                    unsafe { call_game(&mut ro, copy.addr()) };
                    let theirs = capture(&all);
                    fill();
                    unsafe { set_downforce_at(&v) };
                    let mine = capture(&all);
                    let regs_ok = ro.preserved_except(&Regs::SENTINEL, 0);
                    t.check(regs_ok && theirs == mine, || {
                        format!(
                            "control word {word:#x}, seed {seed}, switch {switch}: regs_ok={regs_ok} original {theirs:x?}; rust {mine:x?}"
                        )
                    });
                }
            }
        });
    }
    drop((copy, cells));
    t.finish()
}
