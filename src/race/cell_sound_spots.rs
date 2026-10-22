//! `race::cell_sound_spots` (0x45C6B8): each face of grid cell ecx's record by `race::face_sound_spot` (see `race/sound_spots.rs`).
//!
//! In:  ecx = the cell
//! Out: ebx past the last face, esi the record (or ebx the grid's entry); ecx kept
//!
//! Caller: `race::find_sound_spots` (25 places). Shadow mode: by state. Self-test: a test course
//! (`SpotStage`: faces, records, points, the grid's cells), each cell of the test grid; all put back.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::race::frame_draw::all_regs;
use crate::race::sound_spots::*;
use crate::screens::harness::Left;
use crate::service::harness::{check_state, dispatch_state, state_entry};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::cell_sound_spots", rva(0x45C6B8), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(all_regs(&unsafe { cell_sound_spots(r) }))
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
        (0x42u32, 0u32),
        (0x43, 0),
        (0x44, 0),
        (0x45, 0x20),
        (0x63, 0),
        (0x64, 0),
        (0x65, 0),
        (0x86, 0),
    ] {
        check_state(
            &mut t,
            &format!("cell {n:#x} count {count}"),
            call_original,
            Regs {
                ecx: n,
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
