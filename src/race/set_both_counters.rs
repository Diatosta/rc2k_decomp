//! `race::set_both_counters` (0x43A987): the first two car blocks' +0x71C eax, +0x730 0,
//! +0x72C 0x12C (by their fixed addresses).
//!
//! In:  eax = the value
//! Out: every register preserved
//!
//! See `race/frame.rs`. Six callers in `sub_43A649`. Shadow mode: the Rust version, the
//! dwords put back, the original's, compared. Self-test: the game's two blocks, values,
//! put back.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::race::frame::*;
use crate::screens::harness::Left;
use crate::service::harness::{capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::set_both_counters", rva(0x43A987), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { set_both_counters(r.eax) };
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &both_counter_regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = both_counter_regions();
    let saved = capture(&regions);
    for v in [0u32, 0xCA, 0xFFFF_FFFF] {
        check_state(
            &mut t,
            &format!("value {v:#x}"),
            call_original,
            Regs {
                eax: v,
                ..Regs::SENTINEL
            },
            &|| {
                for &(a, len) in &regions {
                    unsafe { std::ptr::write_bytes(a as *mut u8, 0x5A, len) };
                }
            },
            &regions,
            &rust,
        );
    }
    restore(&regions, &saved);
    t.finish()
}
