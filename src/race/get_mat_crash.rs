//! `race::get_mat_crash` (0x43D43E): the crash matrix from three angles (`getmatcrash`).
//!
//! In:  nothing
//! Out: eax, ebx the z turn's sine and cosine, ecx and edx 0, esi the lit matrix, edi the y turn's, ebp the crash matrix
//!
//! See `race/stage_prefs.rs`. Self-test: the original and the Rust version on the game's
//! variables with values of the test's, put back afterwards.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_write, hook_asm, rva};
use crate::race::stage_prefs::*;
use crate::screens::harness::Left;
use crate::service::harness::{
    Region, at, capture, check_state, dispatch_state, restore, state_entry,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::get_mat_crash", rva(0x43D43E), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn regions() -> Vec<Region> {
    {
        use crate::scene::{matrix::*, view_matrix::*};
        vec![
            (game_addr(rva(0x71A998)) as u32, 12),
            (game_addr(CRASH_TURNS_RVA) as u32, 12),
            (game_addr(TURN_X_RVA) as u32, 0x40),
            (game_addr(TURN_Y_RVA) as u32, 0x40),
            (game_addr(TURN_Z_RVA) as u32, 0x40),
            (game_addr(LIT_RVA) as u32, 0x40),
            (game_addr(CRASH_MATRIX_RVA) as u32, 0x40),
            (game_addr(TURN_X_WORK_RVA) as u32, 0x18),
        ]
    }
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { get_mat_crash(r) }))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
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
    let regions = regions();
    let saved = capture(&regions);
    let _ = (at as fn(usize) -> Region, game_addr as fn(usize) -> usize);
    for (n, angles) in [
        [0u32, 0, 0],
        [0x1234_0000, 0x8000_0000, 0xFFFF_0000],
        [0x4000_1234, 0x0123_4567, 0x89AB_CDEF],
    ]
    .into_iter()
    .enumerate()
    {
        check_state(
            &mut t,
            &format!("angles {n}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                restore(&regions, &saved);
                game_write(rva(0x71A998), angles);
            },
            &regions,
            &rust,
        );
    }
    restore(&regions, &saved);
    t.finish()
}
