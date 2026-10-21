//! `marks::make_noise_curve` (0x4C2247): `init_marks`' step 5, the noise curve walked,
//! smoothed 0xA0 times and ramped, the random tables, the 0x20 records, the quads.
//!
//! In:  nothing
//! Out: eax, ebx, ecx, edx the last values written, esi 0, edi past the records
//!
//! See `marks/setup.rs`. One caller, `marks::init_marks`. Refuses a remainder's divisor of
//! 0, 1 or -1 (the original's `idiv` traps). Self-test: the original and the Rust version
//! on the game's tables with running sums of the test's, put back afterwards.
//!
//! Converted 2026-10-21: the walk, the smoothing, the random tables and the records through
//! `rc2k_core::marks::noise`; the roundings (`fistp`, the spread's floats, the ramp) stay
//! the x87's, as its control word says. Compared exactly as before.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_write, hook_asm, rva};
use crate::marks::setup::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("marks::make_noise_curve", rva(0x4C2247), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { make_noise_curve(r) }?))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &noise_curve_regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = noise_curve_regions();
    let saved = capture(&regions);
    for sum in [0u32, 0x8000_0000, u32::MAX, 0x1234_5678] {
        check_state(
            &mut t,
            &format!("running sum {sum:#x}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                for &(a, n) in &regions {
                    std::ptr::write_bytes(a as *mut u8, 0xAA, n);
                }
                game_write(CURVE_VARS_RVA + 0x14, sum);
            },
            &regions,
            &rust,
        );
    }
    restore(&regions, &saved);
    t.finish()
}
