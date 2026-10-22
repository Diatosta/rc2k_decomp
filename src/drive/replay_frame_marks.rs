//! `drive::replay_frame_marks` (0x41D8A3): the replay frame `dword_5C2530`'s two marks
//! (`unk_963BB4[frame]`, 8 bytes) into `dword_9613A4` .. (in `pusha` / `popa`).
//!
//! In:  nothing
//! Out: every register kept; no flags read
//!
//! Callers: `sub_41D5FB`, `sub_41D846`, `sub_46020A`.
//! Self-test: the original and the Rust version on frames and marks by lot (saved and put
//! back).

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::physics::flow::{a, g, p};
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, check_state, dispatch_state, restore, state_entry};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("drive::replay_frame_marks", rva(0x41D8A3), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The Rust version.
// Converted: `rc2k_core::drive::replay_frames::marks_offset`.
pub fn replay_frame_marks() {
    let at = a(0x963BB4).wrapping_add(rc2k_core::drive::replay_frames::marks_offset(g(0x5C2530)));
    for k in 0..2u32 {
        p(0x9613A4 + 4 * k as usize, unsafe {
            ((at + 4 * k) as *const u32).read_unaligned()
        });
    }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    replay_frame_marks();
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &[(a(0x9613A4), 8)], &rust);
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
    let all: Vec<Region> = vec![(a(0x9613A4), 8), (a(0x5C2530), 4), (a(0x963BB4), 0x40)];
    let saved = capture(&all);
    for case in 0..6u32 {
        let set = || {
            p(0x5C2530, case);
            for k in 0..0x10usize {
                p(0x963BB4 + 4 * k, ((k as u32 + 1) * 0x101) ^ case);
            }
        };
        check_state(
            &mut t,
            &format!("case {case}"),
            call_original,
            Regs::SENTINEL,
            &set,
            &all,
            &rust,
        );
    }
    restore(&all, &saved);
    t.finish()
}
