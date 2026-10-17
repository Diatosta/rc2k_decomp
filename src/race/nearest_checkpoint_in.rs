//! `race::nearest_checkpoint_in` (0x4413F8): of checkpoints ecx up to edx (once at least:
//! the test is after the step, `jb`), the nearest by `race::distance_to_checkpoint`.
//!
//! In:  ecx = the first, edx = the end
//! Out: ebp = the nearest (the later of equals, distances unsigned; 0 if none), edi = its
//!      distance (-1 if none), ecx = past the last; eax, ebx, esi as the last measure
//!      leaves them; the rest preserved; `dword_62400C` the last distance
//!
//! See `race/start_place.rs`. One caller, `race::nearest_checkpoint` (0 .. count).
//! Refuses more than 0x1418 measures or checkpoints, before anything. Shadow mode: the
//! Rust version, the variable put back, the original's, compared. Self-test: faces,
//! points and checkpoints of the test's (`TestLine`), the game's put back; ranges inside,
//! across and past the list, an empty one.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_write, hook_asm, rva};
use crate::race::best_line::{SCRATCH_RVA, TestLine};
use crate::race::start_place::*;
use crate::screens::harness::Left;
use crate::service::harness::{at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "race::nearest_checkpoint_in",
    rva(0x4413F8),
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

fn rust(r: &Regs) -> Result<Left, Refused> {
    let g = unsafe { nearest_checkpoint_in(r) }?;
    Ok(left_changed(r, &g))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &[at(SCRATCH_RVA)], &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let kept = [at(SCRATCH_RVA), at(FROM_X_RVA), at(FROM_Z_RVA)];
    let saved = capture(&kept);
    // Checkpoints 0 and 15 name the same face (equal distances).
    let line = TestLine::new(32, 30, &|k| (k * 7 + 1) % 15, &|_, f| f.fill(0x3C));
    let regions = [at(SCRATCH_RVA)];
    for (from, first, end) in [
        ((0i32, 0i32), 0u32, 30u32),
        ((0x12_3456, -0x7_0000), 0, 30),
        ((-0x30_0000, 0x1_0000), 3, 9),
        ((0, 0), 10, 3),
        ((0, 0), 0, 0),
        ((0, 0), 25, 40),
        ((0, 0), 31, 35),
        ((0, 0), 0xFFFF_FFFE, 2),
    ] {
        check_state(
            &mut t,
            &format!("from {from:x?}, {first:#x} .. {end:#x}"),
            call_original,
            Regs {
                ecx: first,
                edx: end,
                ..Regs::SENTINEL
            },
            &|| unsafe {
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
