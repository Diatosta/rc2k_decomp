//! `drive::race_step` (0x4454D2): the race's ticks for this frame. The clock in tenths
//! (`drive::race_clock_tick`); unless paused (`dword_6138F8`), `drive::race_tick` once for
//! each tenth gone (`dword_613844`, signed), stopping when the players are all out
//! (`race::players_all_out`).
//!
//! In:  nothing
//! Out: ecx the tenths left, the rest as the ticks leave them; no flags read
//!
//! Callers: `drive::race_loop`, `sub_4607A5`. It begins with a call: hooked by a private
//! copy of its 0x34 bytes (four calls relocated) and a hand-made patch.
//! Refuses what its calls refuse. Shadow mode runs the original alone (the clock moves).
//! Self-test: skipped (its parts carry their own; the race in play checks it).

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use crate::physics::flow::g;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("drive::race_step", rva(0x4454D2), thunk, &ORIGINAL)
    .private_copy(0x34)
    .relocate_calls(&[0x0, 0x1A, 0x20, 0x2E])
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The Rust version.
pub unsafe fn race_step(entry: &Regs) -> Result<Left, Refused> {
    let mut r = *entry;
    unsafe { crate::drive::race_clock_tick::race_clock_tick() }.apply(&mut r);
    if g(0x6138F8) == 0 {
        r.ecx = g(0x613844);
        while (r.ecx as i32) > 0 {
            let ecx = r.ecx;
            unsafe { crate::drive::race_tick::race_tick(&r) }?.apply(&mut r);
            r.ecx = ecx;
            if crate::drive::race_parts::players_all_out(&mut r)? {
                break;
            }
            r.ecx -= 1;
        }
    }
    Ok(left_of(&r))
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { race_step(r) } {
        Ok(left) => left.apply(r),
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

fn self_test() -> (usize, usize) {
    let Some(t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    t.skip("its parts carry their own tests; the race in play checks it")
}
