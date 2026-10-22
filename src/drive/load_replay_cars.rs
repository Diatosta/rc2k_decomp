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

use crate::drive::save_replay_cars::{piece_at, pieces};
use crate::errors::Refused;
use crate::framework::{ECX, EDI, ESI, RawHook, Regs, SelfTest, hook_asm, rva};
use crate::physics::flow::{a, p};
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, check_state, dispatch_state, restore, state_entry};
use rc2k_core::drive::replay_frames::{START_DWORDS, start_layout};
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

const SAVED: usize = 0x7172EC; // unk_7172EC

/// The Rust version.
// Converted: `rc2k_core::drive::replay_frames::start_layout`.
pub fn load_replay_cars() -> Left {
    let mut end = 0;
    for (piece, offset, n) in start_layout() {
        let (state, place) = (a(SAVED) + 4 * offset as u32, a(piece_at(piece)));
        unsafe { std::ptr::copy(state as *const u32, place as *mut u32, n) };
        end = place + 4 * n as u32;
    }
    let past = a(SAVED) + 4 * START_DWORDS as u32;
    Left::nothing().with(ECX, 0).with(EDI, end).with(ESI, past)
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    Ok(load_replay_cars())
}

extern "C" fn dispatch(r: &mut Regs) {
    let all: Vec<Region> = pieces().iter().map(|&(x, n)| (a(x), 4 * n)).collect();
    dispatch_state(&HOOK, r, call_original, &all, &rust);
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
    let all: Vec<Region> = {
        let mut v: Vec<Region> = pieces().iter().map(|&(x, n)| (a(x), 4 * n)).collect();
        v.push((a(SAVED), 4 * START_DWORDS));
        v
    };
    let saved = capture(&all);
    for case in 0..2u32 {
        let set = || {
            for j in 0..START_DWORDS {
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
