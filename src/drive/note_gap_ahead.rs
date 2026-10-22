//! `drive::note_gap_ahead` (0x4373A4): this car's checkpoint (esi +0x98) less the lead car's, plus one, into `dword_70FA14` when below it (unsigned): by the look of it the nearest car ahead (a reading).
//!
//! In:  esi a car block
//! Out: edx the difference plus one; the rest kept; no flags read
//!
//! See `drive/note_gaps.rs`. Caller: `sub_43703E` (0x43736B, 0x437397).
//! Self-test: a copy of the original (the lead's checkpoint and the best the test's) on
//! a test car, 7 cases.

use crate::drive::note_gaps::{note_gap_at, test_gap};
use crate::errors::Refused;
use crate::framework::{RawHook, Regs, game_addr, hook_asm, rva};
use crate::screens::harness::Left;
use crate::service::harness::{dispatch_state, state_entry};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("drive::note_gap_ahead", rva(0x4373A4), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const BEST_RVA: usize = rva(0x70FA14);
const LEAD_RVA: usize = rva(0x70F484); // dword_70F484: car block 0's +0x98

/// The Rust version.
pub unsafe fn note_gap_ahead(car: u32) -> Left {
    unsafe {
        note_gap_at(
            car,
            game_addr(LEAD_RVA) as u32,
            game_addr(BEST_RVA) as u32,
            false,
        )
    }
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(unsafe { note_gap_ahead(r.esi) })
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(
        &HOOK,
        r,
        call_original,
        &[(game_addr(BEST_RVA) as u32, 4)],
        &rust,
    );
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}

fn self_test() -> (usize, usize) {
    test_gap(
        &HOOK,
        0x4373A4,
        &[0x8B, 0x96, 0x98, 0x00, 0x00, 0x00],
        0x70FA14,
        false,
    )
}
