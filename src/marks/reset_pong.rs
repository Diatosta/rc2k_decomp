//! `marks::reset_pong` (0x4B7E64): the pong game put back to its start: four (x, y) pairs
//! (the ball and the bats) set from `flt_769182` .., the ball's steps (`g_pongSteps`,
//! 0x768700) 5, the hits (`g_pongHits`) 0. Named `reset_outline` until 2026-10-18.
//!
//! In:  nothing
//! Out: every register kept
//!
//! See `marks/splines.rs`. Callers: `sub_4B7F4F` and `sub_4BF529`. Self-test:
//! the original and the Rust version with the eight floats by lot (a signalling NaN among
//! them, which `fld` makes quiet), put back afterwards.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_write, hook_asm, rva};
use crate::marks::splines::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("marks::reset_pong", rva(0x4B7E64), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const FROM_RVA: usize = rva(0x769182); // flt_769182

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { reset_pong() };
    Ok(left_of(r))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &outline_regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let from: [Region; 1] = [(crate::framework::game_addr(FROM_RVA) as u32, 0x20)];
    let regions = outline_regions();
    let saved = (capture(&from), capture(&regions));
    for seed in 1..=3u32 {
        check_state(
            &mut t,
            &format!("seed {seed}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                let mut s = seed;
                let mut v = [0u32; 8];
                for x in v.iter_mut() {
                    s = s.wrapping_mul(1_103_515_245).wrapping_add(12345);
                    *x = s;
                }
                v[seed as usize] = 0x7F80_0001;
                game_write(FROM_RVA, v);
            },
            &regions,
            &rust,
        );
    }
    restore(&from, &saved.0);
    restore(&regions, &saved.1);
    t.finish()
}
