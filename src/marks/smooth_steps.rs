//! `marks::smooth_steps` (0x4BE928): two cosine-eased steps from ebx to ecx (floats), step
//! eax, into ecx and ebx.
//!
//! In:  eax the step, ebx a, ecx b (floats)
//! Out: ecx, ebx the two steps; eax kept
//!
//! See `marks/turns.rs`. Callers: `sub_4BD122` (twice), `sub_4BEB25` (four). Self-test:
//! the original and the Rust version on values and steps by lot; the two scratch
//! variables put back afterwards.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::marks::turns::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("marks::smooth_steps", rva(0x4BE928), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { smooth_steps(r) }))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &step_regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = step_regions();
    let saved = capture(&regions);
    for seed in 1..=8u32 {
        for step in [0u32, 1, 0x3F, 0x7F, 0x80, u32::MAX] {
            let mut v = [0u32; 2];
            crate::scene::matrix3::test_floats(seed, &mut v);
            check_state(
                &mut t,
                &format!("seed {seed}, step {step:#x}"),
                call_original,
                Regs {
                    eax: step,
                    ebx: v[0],
                    ecx: v[1],
                    ..Regs::SENTINEL
                },
                &|| {},
                &regions,
                &rust,
            );
        }
    }
    restore(&regions, &saved);
    t.finish()
}
