//! `marks::scale_rows3` (0x4BF498): the three rows of three floats at esi each made a set
//! length (`flt_89CA24` .. `flt_89CA2C`).
//!
//! In:  esi the rows
//! Out: every register kept
//!
//! See `marks/turns.rs`. Callers: `sub_4BEFD2` (four). A zero row gives NaNs, as there.
//! Self-test: the original and the Rust version on rows by lot (a zero one among them).
//!
//! Converted 2026-10-21: the hook's Rust version is `turns::scale_rows3_plain`, through
//! `rc2k_core::marks::turns::scale_rows` (plain floats; the x87 version stays for the Rust
//! callers in unconverted code). Shadow mode and the self-test compare within
//! `service::floats`' tolerances (the self-test at both test precisions, exact at double).

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::marks::turns::*;
use crate::race::stage_prefs::left_of;
use crate::scene::matrix::float_places;
use crate::screens::harness::Left;
use crate::service::floats::*;
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
    unsafe { scale_rows3_plain(r.esi) };
    Ok(left_of(r))
}

extern "C" fn dispatch(r: &mut Regs) {
    let regions = [(r.esi, 0x24)];
    let places = float_places(&regions);
    let floats = Floats {
        memory: &places,
        regs: &[],
    };
    dispatch_state_floats(&HOOK, r, call_original, &regions, &floats, &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut rows = [0u32; 9];
    let p = rows.as_mut_ptr() as u32;
    let tolerant_places = float_places(&[(p, 0x24)]);
    let tolerant = Floats {
        memory: &tolerant_places,
        regs: &[],
    };
    at_test_precisions(&tolerant, &mut |word, held| {
        for seed in 0..10u32 {
            check_state_floats(
                &mut t,
                "marks::scale_rows3",
                &format!("control word {word:#x}, seed {seed}"),
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
                held,
                &rust,
            );
        }
    });
    t.finish()
}
