//! `scene::depth_buffer_off` (0x44E3F1): the depth buffer neither used nor written
//! (Direct3D render states 7 = 0, 14 = 0, 23 = 4).
//!
//! In:  nothing
//! Out: eax 4, ebx 0x17; the rest as `scene::set_render_state` leaves them (kept)
//!
//! See `scene/depth_buffer_on.rs`. 11 callers. Self-test: recorded on the layer, nothing
//! drawn.

use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::scene::depth_buffer_on::{check, depth_buffer};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("scene::depth_buffer_off", rva(0x44E3F1), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    unsafe { depth_buffer(r, false) };
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe { depth_buffer(r, false) };
    Ok(())
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    check(&mut t, call_original, &|r| unsafe {
        depth_buffer(r, false)
    });
    t.finish()
}
