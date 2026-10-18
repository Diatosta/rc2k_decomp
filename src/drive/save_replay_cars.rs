//! `drive::save_replay_cars` (0x4600B8): the first 0x1A1 dwords of the four car blocks
//! (`g_carBlockStore`), the race slots (0xF0) and `unk_71BF84` (0x100) copied in that
//! order to `unk_7172EC` (`rep movsd`): the replay's starting state.
//!
//! In:  nothing
//! Out: ecx 0, esi past the last piece, edi past the copy; the rest kept; no flags read
//!
//! Caller: `sub_45FFB6`.
//! Self-test: the original and the Rust version on the pieces filled by pattern (saved
//! and put back).

use crate::errors::Refused;
use crate::framework::{ECX, EDI, ESI, RawHook, Regs, SelfTest, hook_asm, rva};
use crate::physics::flow::{a, p};
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("drive::save_replay_cars", rva(0x4600B8), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const B: usize = 0x70F3EC; // g_carBlockStore (four blocks of 0xFE0)
const SAVED: usize = 0x7172EC; // unk_7172EC
/// The pieces in order: (where, dwords).
const PIECES: [(usize, usize); 6] = [
    (B, 0x1A1),
    (B + 0xFE0, 0x1A1),
    (B + 0x1FC0, 0x1A1),
    (B + 0x2FA0, 0x1A1),
    (0x71BAC8, 0xF0),  // g_raceSlots
    (0x71BF84, 0x100), // unk_71BF84
];
const TOTAL: usize = 4 * 0x1A1 + 0xF0 + 0x100;

/// The Rust version.
pub fn save_replay_cars() -> Left {
    let mut to = a(SAVED);
    for (from, n) in PIECES {
        unsafe { std::ptr::copy(a(from) as *const u32, to as *mut u32, n) };
        to += 4 * n as u32;
    }
    let (last, n) = PIECES[5];
    Left::nothing()
        .with(ECX, 0)
        .with(ESI, a(last) + 4 * n as u32)
        .with(EDI, to)
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    Ok(save_replay_cars())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &[(a(SAVED), 4 * TOTAL)], &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let all: Vec<Region> = {
        let mut v: Vec<Region> = PIECES.iter().map(|&(x, n)| (a(x), 4 * n)).collect();
        v.push((a(SAVED), 4 * TOTAL));
        v
    };
    let saved = capture(&all);
    for case in 0..2u32 {
        let set = || {
            for (k, &(x, n)) in PIECES.iter().enumerate() {
                for j in 0..n {
                    p(x + 4 * j, (k * 0x1000 + j) as u32 ^ case);
                }
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
