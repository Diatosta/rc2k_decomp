//! `marks::set_line_turn` (0x4BF432): the 2 x 2 turn of the line at esi (dx, minus dy) into
//! `flt_82E396` .. `flt_82E3A6`.
//!
//! In:  esi the line (+0, +4 to +0xC, +0x10)
//! Out: every register kept
//!
//! See `marks/turns.rs`. Callers: `sub_4BEFD2` (twice). Self-test: the original and the
//! Rust version on lines by lot, put back afterwards.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::marks::turns::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("marks::set_line_turn", rva(0x4BF432), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { set_line_turn(r.esi, false) };
    Ok(left_of(r))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &turn_regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = turn_regions();
    let saved = capture(&regions);
    let mut line = [0u32; 5];
    let p = line.as_mut_ptr() as u32;
    for seed in 1..=10u32 {
        check_state(
            &mut t,
            &format!("seed {seed}"),
            call_original,
            Regs {
                esi: p,
                ..Regs::SENTINEL
            },
            &|| {
                let mut v = [0u32; 5];
                crate::scene::matrix3::test_floats(seed, &mut v);
                unsafe { (p as *mut [u32; 5]).write(v) };
            },
            &regions,
            &rust,
        );
    }
    restore(&regions, &saved);
    t.finish()
}
