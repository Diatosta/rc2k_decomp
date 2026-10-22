//! `scene::depth_test_on` (0x44E3E1): Direct3D render state 14 (writing to the depth buffer)
//! 1.
//!
//! In:  nothing
//! Out: eax 1, ebx 0xE; the rest as `scene::set_render_state` leaves them (kept)
//!
//! One call of `scene::set_render_state`. 15 callers. Self-test: the original and the Rust
//! version recorded on the layer, nothing drawn.

use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::scene::depth_buffer_on::check;
use crate::scene::set_render_state::set_render_state;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("scene::depth_test_on", rva(0x44E3E1), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The Rust version (see the header).
pub unsafe fn depth_test_on(r: &mut Regs) {
    unsafe { set_render_state(r, 1, 0xE) };
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    unsafe { depth_test_on(r) };
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe { depth_test_on(r) };
    Ok(())
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    check(&mut t, call_original, &|r| unsafe { depth_test_on(r) });
    t.finish()
}
