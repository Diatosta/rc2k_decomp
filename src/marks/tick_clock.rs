//! `marks::tick_clock` (0x4D2BDC): the marks' clock, started or moved on by
//! `g_sceneClock`.
//!
//! In:  nothing
//! Out: the first time every register preserved; then eax a sixteenth of the ticks, ebx
//!      the clock before, esi `g_marksSettings`
//!
//! See `marks/setup.rs`. Callers: `marks::init_marks` (step 15) and `marks::run_frame`
//! (each frame). Self-test: the original and the Rust version on the game's variables with
//! clocks, settings and a start of the test's, put back afterwards.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_write, hook_asm, rva};
use crate::marks::setup::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("marks::tick_clock", rva(0x4D2BDC), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn regions() -> Vec<Region> {
    vec![(game_addr(CLOCK_VARS_RVA) as u32, CLOCK_VARS_LEN)]
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { tick_clock(r) }))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = regions();
    let inputs = [at(CLOCK_RVA), at(SETTINGS_RVA)];
    let saved = capture(&regions);
    let saved_inputs = capture(&inputs);
    let mut settings = [0xAAAA_AAAAu32; 8];
    let object = settings.as_mut_ptr() as u32;
    for started in [0u32, 1] {
        for mode in [1u32, 0, 2] {
            for (last, clock) in [
                (0u32, 0u32),
                (100, 101),
                (100, 103),
                (0x1000, 0x1011),
                (5, 0x12_3456),
                (0x10, 0x8),
                (u32::MAX - 3, 4),
            ] {
                check_state(
                    &mut t,
                    &format!("started {started}, +0x10 {mode}, last {last:#x}, clock {clock:#x}"),
                    call_original,
                    Regs::SENTINEL,
                    &|| unsafe {
                        for &(a, n) in &regions {
                            std::ptr::write_bytes(a as *mut u8, 0xAA, n);
                        }
                        ((object + 0x10) as *mut u32).write_volatile(mode);
                        game_write(CLOCK_VARS_RVA, [started, last]);
                        game_write(CLOCK_RVA, clock);
                        game_write(SETTINGS_RVA, object);
                    },
                    &regions,
                    &rust,
                );
            }
        }
    }
    let _ = &mut settings;
    restore(&inputs, &saved_inputs);
    restore(&regions, &saved);
    t.finish()
}
