//! `marks::make_wave_tables` (0x4C4E2F): `init_marks`' step 2, once: the 0x11 wave
//! levels made floats and the 0x140 sines.
//!
//! In:  nothing
//! Out: eax `g_marksWaveDone`'s address, edi past the sines; done before: every register
//!      preserved
//!
//! See `marks/setup.rs`. One caller, `marks::init_marks`. Refuses a mark that is neither 0
//! nor `nullsub_23`'s address (the original jumps to it). Self-test: the original and the
//! Rust version on the game's tables with levels of the test's, the mark 0 and set; the
//! refusal; put back afterwards.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_write, hook_asm, rva};
use crate::marks::setup::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("marks::make_wave_tables", rva(0x4C4E2F), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn regions() -> Vec<Region> {
    vec![
        at(WAVE_DONE_RVA),
        (game_addr(WAVE_LEVELS_RVA) as u32, 4 * WAVE_LEVELS as usize),
        (game_addr(SINES_RVA) as u32, 4 * SINES as usize),
    ]
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { make_wave_tables(r) }?))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

/// The game's levels as they are in RAL.EXE.
const LEVELS: [i32; 17] = [
    0, 9, 43, 98, 180, 191, 198, 231, 265, 313, 337, 348, 350, 378, 385, 434, 512,
];

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = regions();
    let saved = capture(&regions);
    let others: [i32; 17] = std::array::from_fn(|n| (n as i32 - 8) * 0x0123_4567);
    let extremes: [i32; 17] = std::array::from_fn(|n| match n % 4 {
        0 => i32::MIN,
        1 => i32::MAX,
        2 => -1,
        _ => 0x7F,
    });
    let target = game_addr(rva(0x4C4EA0)) as u32;
    for (what, levels, done) in [
        ("the game's levels", LEVELS, 0),
        ("others", others, 0),
        ("extremes", extremes, 0),
        ("done before", LEVELS, target),
    ] {
        check_state(
            &mut t,
            what,
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                for &(a, n) in &regions {
                    std::ptr::write_bytes(a as *mut u8, 0xAA, n);
                }
                game_write(WAVE_LEVELS_RVA, levels);
                game_write(WAVE_DONE_RVA, done);
            },
            &regions,
            &rust,
        );
    }
    unsafe { game_write(WAVE_DONE_RVA, 0x1234u32) };
    t.check(
        unsafe { make_wave_tables(&Regs::SENTINEL) }.is_err(),
        || "a mark of 0x1234: the Rust version went on".into(),
    );
    restore(&regions, &saved);
    t.finish()
}
