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
use crate::service::harness::{Region, capture, check_state, dispatch_state, restore, state_entry};
use rc2k_core::drive::replay_frames::{START_DWORDS, START_PIECES, StartPiece, start_layout};
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

/// Where a piece of the starting state is.
pub fn piece_at(piece: StartPiece) -> usize {
    match piece {
        StartPiece::Car(k) => B + 0xFE0 * k as usize,
        StartPiece::RaceSlots => 0x71BAC8, // g_raceSlots
        StartPiece::Extra => 0x71BF84,     // unk_71BF84
    }
}

/// The pieces in order: (where, dwords).
pub fn pieces() -> Vec<(usize, usize)> {
    START_PIECES
        .iter()
        .map(|&(x, n)| (piece_at(x), n))
        .collect()
}

/// The Rust version.
// Converted: `rc2k_core::drive::replay_frames::start_layout`.
pub fn save_replay_cars() -> Left {
    let mut end = 0;
    for (piece, offset, n) in start_layout() {
        let (state, place) = (a(SAVED) + 4 * offset as u32, a(piece_at(piece)));
        unsafe { std::ptr::copy(place as *const u32, state as *mut u32, n) };
        end = place + 4 * n as u32;
    }
    let past = a(SAVED) + 4 * START_DWORDS as u32;
    Left::nothing().with(ECX, 0).with(ESI, end).with(EDI, past)
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    Ok(save_replay_cars())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(
        &HOOK,
        r,
        call_original,
        &[(a(SAVED), 4 * START_DWORDS)],
        &rust,
    );
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
            for (k, &(x, n)) in pieces().iter().enumerate() {
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
