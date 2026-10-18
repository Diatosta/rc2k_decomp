//! `physics::drive_force` (0x43CB14): the engine's drive. `physics::engine_torque_or_cut`;
//! `flt_614740` = (torque + `flt_614744`) x +0x638 (`unk_71A610`, `fild`) / 256 -
//! `flt_614744`; `flt_614750` = it x `flt_5CB968` / the gear's ratio / 10000, or 0 with
//! +0x1D0 (`unk_71A1A8`) set; `physics::drive_torque_share`.
//!
//! In:  nothing
//! Out: ebx the gear, and what the calls leave; no flags read (the caller goes on with a
//!      call)
//!
//! Caller: `sub_43B635` (0x43B732), every physics step.
//! Refuses (before anything) a gear outside -1 ..= 7.
//! Self-test: `physics::engine_torque`'s (`test_engine`).

use crate::errors::{BadInput, Refused};
use crate::framework::{EBX, RawHook, Regs, SelfTest, hook_asm, rva};
use crate::math::x87_run::{mul_div2, sum_times_int_over_minus};
use crate::physics::drive_torque_share::drive_torque_share;
use crate::physics::engine_torque::{g, p, regions, test_engine};
use crate::physics::engine_torque_or_cut::engine_torque_or_cut;
use crate::screens::harness::Left;
use crate::service::harness::dispatch_state;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("physics::drive_force", rva(0x43CB14), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The Rust version.
pub unsafe fn drive_force() -> Result<Left, Refused> {
    let gear = g(0x71A1AC);
    if !(-1..=7).contains(&(gear as i32)) {
        return Err(BadInput("a gear outside g_gearRatios (-1 ..= 7)").into());
    }
    let left = unsafe { engine_torque_or_cut() };
    p(
        0x614740,
        sum_times_int_over_minus(g(0x614748), g(0x614744), g(0x71A610) as i32, g(0x5C5E2C)),
    );
    p(
        0x614750,
        mul_div2(
            g(0x614740),
            g(0x5CB968),
            g(0x6140C0 + 4 * gear as usize),
            g(0x5C5E34),
        ),
    );
    if g(0x71A1A8) != 0 {
        p(0x614750, 0);
    }
    let eax = unsafe { drive_torque_share() };
    Ok(left.with(EBX, gear).with(crate::framework::EAX, eax))
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { drive_force() }
}

extern "C" fn dispatch(r: &mut Regs) {
    let mut regions = regions();
    let a = |x: usize| crate::framework::game_addr(rva(x)) as u32;
    regions.extend([(a(0x614758), 4), (a(0x624020), 4), (a(0x62401C), 4)]);
    dispatch_state(&HOOK, r, call_original, &regions, &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    test_engine(&mut t, call_original, &rust);
    t.finish()
}
