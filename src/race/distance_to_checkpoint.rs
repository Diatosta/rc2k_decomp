//! `race::distance_to_checkpoint` (0x44141D): how far checkpoint ecx's face's first point
//! is in x and z from (`dword_639824`, `dword_63982C`), into `dword_62400C`; -1 for a
//! checkpoint outside 0 .. `g_checkpointCount` - 1 (sign tests).
//!
//! In:  ecx = the checkpoint
//! Out: `dword_62400C`; with a checkpoint eax = the z difference, ebx and esi = the point's
//!      number and address; the rest preserved
//!
//! See `race/start_place.rs`. One caller, `race::nearest_checkpoint_in` (0x4413F8). The
//! length: `fild` / `fmul st, st` twice, `faddp`, `fsqrt`, `fistp` (`x87::int_hypot`).
//! Its first bytes hold a `js`: private copy (the whole function, no calls), manual
//! patch. Refuses a checkpoint inside the count but past the list (0x1418). Shadow mode: the
//! Rust version, the variable put back, the original's, compared. Self-test: faces,
//! points and checkpoints of the test's (`TestLine`), the game's put back.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_write, hook_asm, rva};
use crate::race::best_line::{CHECK_COUNT_RVA, SCRATCH_RVA, TestLine};
use crate::race::start_place::*;
use crate::screens::harness::Left;
use crate::service::harness::{at, capture, check_state, dispatch_state, restore, state_entry};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "race::distance_to_checkpoint",
    rva(0x44141D),
    thunk,
    &ORIGINAL,
)
.private_copy(0x72)
.manual_patch()
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(r: &Regs) -> Result<Left, Refused> {
    let g = unsafe { distance_to_checkpoint(r) }?;
    Ok(left_changed(r, &g))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &[at(SCRATCH_RVA)], &rust);
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
    let kept = [at(SCRATCH_RVA), at(FROM_X_RVA), at(FROM_Z_RVA)];
    let saved = capture(&kept);
    let line = TestLine::new(32, 30, &|k| (k * 7 + 1) % 32, &|_, f| f.fill(0x3C));
    let regions = [at(SCRATCH_RVA)];
    for (count, from, check) in [
        (30u32, (0i32, 0i32), 0u32),
        (30, (0, 0), 29),
        (30, (0x12_3456, -0x7_0000), 5),
        (30, (-0x40_0000, 0x1_5555), 0),
        (30, (0x7FFF_0000, -0x7FFF_0000), 17),
        (30, (0, 0), 30),
        (30, (0, 0), u32::MAX),
        (30, (0, 0), 0x8000_0000),
        (0, (0, 0), 0),
        (0x8000_0002, (5, 5), 3),
    ] {
        check_state(
            &mut t,
            &format!("count {count:#x}, from {from:x?}, checkpoint {check:#x}"),
            call_original,
            Regs {
                ecx: check,
                ..Regs::SENTINEL
            },
            &|| unsafe {
                game_write(CHECK_COUNT_RVA, count);
                game_write(FROM_X_RVA, from.0);
                game_write(FROM_Z_RVA, from.1);
                game_write(SCRATCH_RVA, 0x5A5A_5A5Au32);
            },
            &regions,
            &rust,
        );
    }
    drop(line);
    restore(&kept, &saved);
    t.finish()
}
