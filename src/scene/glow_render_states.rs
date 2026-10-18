//! `scene::glow_render_states` (0x44E44D): three of Direct3D's render states, for what is
//! drawn glowing (the sky's sun, the views' backdrop): 7 = 1 (the depth test on), 0xE = 1
//! (depth writing on), 0x17 = 8 (the comparison: always), each by `scene::set_render_state`.
//!
//! In:  nothing
//! Out: eax = 8, ebx = 0x17 (the last state's); the rest kept; no flags read
//!
//! Callers: `race::draw_sky`, `race::draw_view_backdrop`. Self-test: the original and the
//! Rust version recorded on the layer, nothing drawn.

use crate::d3d::recorder::record;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("scene::glow_render_states", rva(0x44E44D), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const STATES: [(u32, u32); 3] = [(1, 7), (1, 0xE), (8, 0x17)];

/// The Rust version (see the header): the registers it leaves on `r`.
pub unsafe fn glow_render_states(r: &mut Regs) {
    for (value, state) in STATES {
        unsafe { crate::scene::set_render_state::set_render_state(r, value, state) };
    }
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    unsafe { glow_render_states(r) };
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let before = Regs::SENTINEL;
    let (theirs, their_calls) = record(true, || {
        let mut r = before;
        unsafe { call_original(&mut r) };
        r
    });
    let (mine, my_calls) = record(true, || {
        let mut r = before;
        unsafe { glow_render_states(&mut r) };
        r
    });
    let regs = |r: &Regs| [r.eax, r.ebx, r.ecx, r.edx, r.esi, r.edi, r.ebp];
    t.check(
        regs(&theirs) == regs(&mine) && their_calls == my_calls,
        || {
            format!(
                "original {:x?} {their_calls:x?}; rust {:x?} {my_calls:x?}",
                regs(&theirs),
                regs(&mine)
            )
        },
    );
    t.finish()
}
