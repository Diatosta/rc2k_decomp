//! `race::start_kept_reflections` (0x451864): the 0x400 kept reflection normals
//! (`g_keptReflectionNormals`) set to -999 and `g_reflectionsKept` 1, before an object is
//! drawn whose reflections are worked out once (see `race/frame_draw.rs`).
//!
//! In:  nothing
//! Out: eax -999, ecx 0, edi past the normals (a `call` follows)
//!
//! Callers: `race::draw_slot_object`, `sub_4616AA` (0x461DF2). Shadow mode: by state.
//! Self-test: the normals and the switch filled with 0xAA first; all put back.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, hook_asm, rva};
use crate::race::frame_draw::*;
use crate::screens::harness::Left;
use crate::service::harness::{
    Region, at, capture, check_state, dispatch_state, restore, state_entry,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "race::start_kept_reflections",
    rva(0x451864),
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

pub fn kept_regions() -> Vec<Region> {
    vec![
        (game_addr(KEPT_NORMALS_RVA) as u32, 0x1000),
        at(REFLECTIONS_KEPT_RVA),
    ]
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    Ok(unsafe { start_kept_reflections() })
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &kept_regions(), &rust);
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
    let regions = kept_regions();
    let saved = capture(&regions);
    check_state(
        &mut t,
        "filled",
        call_original,
        Regs::SENTINEL,
        &|| restore(&regions, &vec![0xAA; saved.len()]),
        &regions,
        &rust,
    );
    restore(&regions, &saved);
    t.finish()
}
