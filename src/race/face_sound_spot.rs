//! `race::face_sound_spot` (0x45C6ED): face ebx as a sound spot (kind 0xFF, object 0x3F, sound 1 .. 0x15, room left) (see `race/sound_spots.rs`).
//!
//! In:  ebx = the face
//! Out: esi, edi, eax .. ecx as it leaves them (inside its caller's `pusha`)
//!
//! Caller: `race::cell_sound_spots`. Shadow mode: by state. Self-test: a test course
//! (`SpotStage`: faces, records, points, the grid's cells), every test face, the spots full and not; all put back.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::race::frame_draw::all_regs;
use crate::race::sound_spots::*;
use crate::screens::harness::Left;
use crate::service::harness::{check_state, dispatch_state, state_entry};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::face_sound_spot", rva(0x45C6ED), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(all_regs(&unsafe { face_sound_spot(r) }))
}

extern "C" fn dispatch(r: &mut Regs) {
    let regions = SpotStage::live_regions();
    dispatch_state(&HOOK, r, call_original, &regions, &rust);
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
    let stage = SpotStage::new();
    let regions = stage.regions();
    for (n, count) in [
        (0u32, 0u32),
        (1, 0),
        (2, 3),
        (3, 0x23),
        (3, 0x24),
        (4, 0),
        (6, 1),
        (8, 0),
        (9, 0),
        (13, 2),
        (21, 5),
    ] {
        check_state(
            &mut t,
            &format!("face {n:#x} count {count}"),
            call_original,
            Regs {
                ebx: n,
                ..Regs::SENTINEL
            },
            &|| stage.prepare(4, count),
            &regions,
            &rust,
        );
    }
    stage.end();
    t.finish()
}
