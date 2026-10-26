//! `race::first_check_checkpoint` (0x446280): the first checkpoint from 1 whose face has a
//! check (+0x26 not 0), up to `g_lastCheckpoint` (0x623644, not included).
//!
//! In:  nothing
//! Out: eax = that checkpoint, 0 for none or with `g_checkpointCount` - 2 negative (then
//!      nothing else changed); else edx = the checkpoint (`g_lastCheckpoint` for none),
//!      ebx and esi = the last face looked at (number, address); the rest preserved
//!
//! See `race/race_init.rs`. One caller, `sub_4464AD` (the camera's start, under
//! `sub_446463`, race init's first step; not with `dword_62389C` 4). Refuses a look that
//! would run past the checkpoint list (`g_lastCheckpoint` 0 or 1 with no check found: the
//! original runs on). Shadow mode: the Rust version and the original, registers compared
//! (it writes nothing). Self-test: faces and checkpoints of the test's (`TestLine`), the
//! game's put back; the refusal on the Rust version alone.

use crate::errors::Refused;
use crate::framework::{EAX, EBX, EDX, ESI, RawHook, Regs, SelfTest, game_write, hook_asm, rva};
use crate::race::best_line::{CHECK_COUNT_RVA, MOST_CHECKS, TestLine};
use crate::race::race_init::*;
use crate::screens::harness::Left;
use crate::service::harness::{at, capture, check_state, dispatch_state, restore, state_entry};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "race::first_check_checkpoint",
    rva(0x446280),
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
    let g = unsafe { first_check_checkpoint(r) }?;
    Ok(Left::nothing()
        .with(EAX, g.eax)
        .with(EDX, g.edx)
        .with(EBX, g.ebx)
        .with(ESI, g.esi))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &[], &rust);
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
    let saved = capture(&[at(LAST_CHECK_RVA)]);
    // Faces 0 .. 31; checkpoint k names face (k * 5 + 2) % 32; faces 27 and 14 have checks
    // (checkpoints 5 and 28).
    let line = TestLine::new(32, 30, &|k| (k * 5 + 2) % 32, &|n, f| {
        f.fill(0);
        f[0x26] = match n {
            27 => 1,
            14 => 0x80,
            _ => 0,
        };
        f[0x27] = 0xA5;
    });
    let regions = [line.faces.region()];
    for (count, last) in [
        (0u32, 1u32),
        (1, 0),
        (0x8000_0001, 0),
        (2, 2),
        (2, 3),
        (30, 5),
        (30, 6),
        (30, 14),
        (30, 30),
        (3, 30),
        (0x7FFF_FFFF, 7),
    ] {
        check_state(
            &mut t,
            &format!("count {count:#x}, last {last}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                game_write(CHECK_COUNT_RVA, count);
                game_write(LAST_CHECK_RVA, last);
            },
            &regions,
            &rust,
        );
    }
    // Checkpoint 5 has a check, so with `g_lastCheckpoint` 1 it is found all the same.
    unsafe {
        game_write(CHECK_COUNT_RVA, 30u32);
        game_write(LAST_CHECK_RVA, 1u32);
        let found = first_check_checkpoint(&Regs::SENTINEL).map(|g| g.eax);
        t.check(found == Ok(5), || {
            format!("count 30, last 1: rust {found:?}, expected Ok(5)")
        });
    }
    drop(line);
    // No check in the whole list and `g_lastCheckpoint` 1: the original would run on, the
    // Rust version refuses (the Rust version alone).
    let empty = TestLine::new(32, MOST_CHECKS, &|k| k % 32, &|_, f| f.fill(0));
    unsafe {
        game_write(CHECK_COUNT_RVA, 30u32);
        game_write(LAST_CHECK_RVA, 1u32);
        let refused = first_check_checkpoint(&Regs::SENTINEL).is_err();
        t.check(refused, || "no check, last 1: not refused".to_string());
    }
    drop(empty);
    restore(&[at(LAST_CHECK_RVA)], &saved);
    t.finish()
}
