//! `scene::set_render_state` (0x44EB72): one of Direct3D's render states.
//!
//! In:  eax = the value, ebx = the state
//! Out: as `d3d::set_render_state` leaves them: every register preserved
//!
//! A call of `d3d::set_render_state` and nothing more. 208 call sites.
//! Manual patch with a private copy: the function is that call.
//! Self-test: the original and the Rust version recorded on the layer, nothing drawn.

use crate::d3d::recorder::record;
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, hook_asm, rva};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("scene::set_render_state", rva(0x44EB72), thunk, &ORIGINAL)
    .private_copy(6)
    .relocate_calls(&[0])
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const LAYER_RVA: usize = rva(0x41AB5B); // d3d::set_render_state

// ---- Rust version ----

/// `d3d::set_render_state` with the value and the state, the other registers as given.
pub unsafe fn set_render_state(r: &mut Regs, value: u32, state: u32) {
    r.eax = value;
    r.ebx = state;
    unsafe { call_game(r, game_addr(LAYER_RVA)) };
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    let (value, state) = (r.eax, r.ebx);
    unsafe { set_render_state(r, value, state) };
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    for (value, state) in [(1u32, 0x1Bu32), (0, 0x0E), (6, 0x14), (0xFFFF_FFFF, 0x3F)] {
        let before = Regs {
            eax: value,
            ebx: state,
            ..Regs::SENTINEL
        };
        let (theirs, their_calls) = record(true, || {
            let mut r = before;
            unsafe { call_original(&mut r) };
            r
        });
        let (mine, my_calls) = record(true, || {
            let mut r = before;
            unsafe { set_render_state(&mut r, value, state) };
            r
        });
        t.check(
            theirs.preserved_except(&before, 0)
                && mine.preserved_except(&before, 0)
                && their_calls == my_calls,
            || {
                format!(
                    "value {value:#x}, state {state:#x}: calls {their_calls:x?} / {my_calls:x?}"
                )
            },
        );
    }
    t.finish()
}
