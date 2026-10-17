//! `marks::scale_rows3` (0x4BF498): the three rows of three floats at esi each made a set
//! length (`flt_89CA24` .. `flt_89CA2C`).
//!
//! In:  esi the rows
//! Out: every register kept
//!
//! See `marks/turns.rs`. Callers: `sub_4BEFD2` (four). A zero row gives NaNs, as there.
//! Self-test: the original and the Rust version on rows by lot (a zero one among them).

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::marks::turns::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{check_state, dispatch_state};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("marks::scale_rows3", rva(0x4BF498), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { scale_rows3(r.esi) };
    Ok(left_of(r))
}

extern "C" fn dispatch(r: &mut Regs) {
    let regions = [(r.esi, 0x24)];
    dispatch_state(&HOOK, r, call_original, &regions, &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut rows = [0u32; 9];
    let p = rows.as_mut_ptr() as u32;
    for seed in 0..10u32 {
        check_state(
            &mut t,
            &format!("seed {seed}"),
            call_original,
            Regs {
                esi: p,
                ..Regs::SENTINEL
            },
            &|| {
                let mut v = [0u32; 9];
                crate::scene::matrix3::test_floats(seed + 1, &mut v);
                if seed == 0 {
                    v[3..6].fill(0);
                }
                unsafe { (p as *mut [u32; 9]).write(v) };
            },
            &[(p, 0x24)],
            &rust,
        );
    }
    t.finish()
}
