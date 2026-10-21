//! `drive::unpack_inputs` (0x409338): a packed input word (`drive::pack_inputs`) taken
//! apart: eax bits 4 .. 12, edx bits 13 .. 21, ecx the top ten (`sar` 22).
//!
//! In:  eax the packed word
//! Out: eax, ecx, edx the words; the rest kept; no flags read
//!
//! Callers: `drive::play_inputs`, `drive::record_inputs`.
//! Self-test: the original and the Rust version on words by lot.

use crate::errors::Refused;
use crate::framework::{EAX, ECX, EDX, RawHook, Regs, SelfTest, hook_asm, rva};
use crate::physics::flow::{a, g, p};
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("drive::unpack_inputs", rva(0x409338), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The Rust version: [eax, ecx, edx].
// Converted: `rc2k_core::drive::inputs::unpack_inputs`.
pub use rc2k_core::drive::inputs::unpack_inputs;

fn rust(r: &Regs) -> Result<Left, Refused> {
    let [eax, ecx, edx] = unpack_inputs(r.eax);
    Ok(Left::nothing().with(EAX, eax).with(ECX, ecx).with(EDX, edx))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &[], &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let all: Vec<Region> = vec![];
    let saved = capture(&all);
    for case in 0..16u32 {
        let set = || {
            let _ = (a, g, p);
        };
        check_state(
            &mut t,
            &format!("case {case}"),
            call_original,
            Regs {
                eax: case.wrapping_mul(0x9E37_79B9),
                ..Regs::SENTINEL
            },
            &set,
            &all,
            &rust,
        );
    }
    restore(&all, &saved);
    t.finish()
}
