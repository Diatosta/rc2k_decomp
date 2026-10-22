//! `drive::pack_inputs` (0x409328): four input words packed into one: eax | ((ecx << 9 |
//! edx) << 9), << 4 (as the original's `shl`s, high bits lost).
//!
//! In:  eax, ecx, edx the words (ebx not used)
//! Out: eax the packed word; the rest kept; no flags read
//!
//! Caller: `drive::record_inputs`.
//! Self-test: the original and the Rust version on words by lot.

use crate::errors::Refused;
use crate::framework::{EAX, RawHook, Regs, SelfTest, hook_asm, rva};
use crate::physics::flow::{a, g, p};
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, check_state, dispatch_state, restore, state_entry};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("drive::pack_inputs", rva(0x409328), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The Rust version.
// Converted: `rc2k_core::drive::inputs::pack_inputs`.
pub use rc2k_core::drive::inputs::pack_inputs;

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(Left::nothing().with(EAX, pack_inputs(r.eax, r.ecx, r.edx)))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &[], &rust);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
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
                eax: case.wrapping_mul(0x9E37_79B9) >> (case % 8),
                ecx: case.wrapping_mul(0x85EB_CA6B),
                edx: case * 0x77,
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
