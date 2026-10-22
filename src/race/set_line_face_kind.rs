//! `race::set_line_face_kind` (0x44574A): the kind of the faces the best line runs
//! through, by the stage's direction.
//!
//! In:  nothing
//! Out: ebx = the kind; the rest preserved
//!
//! See `race/best_line.rs`. One caller, `sub_4458A7`. Self-test: the original and the Rust
//! version on the game's variables, put back afterwards.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_write, hook_asm, rva};
use crate::race::best_line::*;
use crate::screens::harness::Left;
use crate::service::harness::{at, capture, check_state, dispatch_state, restore, state_entry};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::set_line_face_kind", rva(0x44574A), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(crate::race::stage_prefs::left_of(&unsafe {
        set_line_face_kind(r)
    }))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &[at(LINE_KIND_RVA)], &rust);
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
    let regions = [at(LINE_KIND_RVA), at(DIRECTION_RVA)];
    let saved = capture(&regions);
    for direction in [0u32, 1, 2, u32::MAX] {
        check_state(
            &mut t,
            &format!("direction {direction:#x}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                game_write(DIRECTION_RVA, direction);
                game_write(LINE_KIND_RVA, 0x77u32);
            },
            &regions,
            &rust,
        );
    }
    restore(&regions, &saved);
    t.finish()
}
