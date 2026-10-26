//! `race::end_kept_reflections` (0x451881): `g_reflectionsKept` 0 after the drawing (see
//! `race::start_kept_reflections`).
//!
//! In:  nothing
//! Out: every register preserved
//!
//! Callers: `race::draw_slot_object`, `sub_4616AA` (0x461DFC). Shadow mode: by state.
//! Self-test: the switch 1 and 0xAAAAAAAA first; put back.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_write, hook_asm, rva};
use crate::race::frame_draw::*;
use crate::screens::harness::Left;
use crate::service::harness::{at, capture, check_state, dispatch_state, restore, state_entry};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "race::end_kept_reflections",
    rva(0x451881),
    thunk,
    &ORIGINAL,
)
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { end_kept_reflections() };
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &[at(REFLECTIONS_KEPT_RVA)], &rust);
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
    let regions = [at(REFLECTIONS_KEPT_RVA)];
    let saved = capture(&regions);
    for v in [1u32, 0xAAAA_AAAA] {
        check_state(
            &mut t,
            &format!("switch {v:#x}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe { game_write(REFLECTIONS_KEPT_RVA, v) },
            &regions,
            &rust,
        );
    }
    restore(&regions, &saved);
    t.finish()
}
