//! `drive::load_replay_cars` (0x46010D): `drive::save_replay_cars` the other way: the
//! replay's starting state at `unk_7172EC` copied back into the four car blocks, the race
//! slots and `unk_71BF84`.
//!
//! In:  nothing
//! Out: ecx 0, esi past the copy, edi past the last piece; the rest kept; no flags read
//!
//! Callers: `sub_460049`, `sub_46062E` (twice).
//! Self-test: the original and the Rust version on the copy filled by pattern (saved and
//! put back).

use crate::errors::Refused;
use crate::framework::{ECX, EDI, ESI, RawHook, Regs, SelfTest, hook_asm, rva};
use crate::physics::flow::{a, p};
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("drive::load_replay_cars", rva(0x46010D), thunk, &ORIGINAL).self_test(self_test);

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
pub fn load_replay_cars() -> Left {
    let mut from = a(SAVED);
    for (to, n) in PIECES {
        unsafe { std::ptr::copy(from as *const u32, a(to) as *mut u32, n) };
        from += 4 * n as u32;
    }
    let (last, n) = PIECES[5];
    Left::nothing()
        .with(ECX, 0)
        .with(ESI, from)
        .with(EDI, a(last) + 4 * n as u32)
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    Ok(load_replay_cars())
}

extern "C" fn dispatch(r: &mut Regs) {
    let all: Vec<Region> = PIECES.iter().map(|&(x, n)| (a(x), 4 * n)).collect();
    dispatch_state(&HOOK, r, call_original, &all, &rust);
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
            for j in 0..TOTAL {
                p(SAVED + 4 * j, j as u32 ^ (case << 20));
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
