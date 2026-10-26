//! `race::players_all_out` (0x445506): carry when every player is out of the race (+0xB0
//! not 0): the chosen car with `dword_52A910` 1, else each of `dword_63DB94` players (none
//! gives no carry).
//!
//! In:  nothing
//! Out: carry set when all are out; every register preserved (`pusha` / `popa`)
//!
//! See `race/frame.rs`. Callers: `sub_444CEE` and `sub_4454D2` (stream B's; each `jb` after
//! it: dispatch sets the carry). Refuses, before anything, a chosen car past 7 or more than
//! 8 players. Shadow mode: the Rust version and the original, carry compared (nothing
//! written). Self-test: the game's blocks' +0xB0 set, one chosen car and 0 .. 8 players,
//! put back.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_write, hook_asm, rva};
use crate::race::frame::*;
use crate::screens::harness::Left;
use crate::service::harness::{
    Region, at, capture, check_state, dispatch_state, restore, state_entry,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::players_all_out", rva(0x445506), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(_: &Regs) -> Result<Left, Refused> {
    Ok(Left::carry(unsafe { players_all_out() }?))
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
    let blocks: Vec<Region> = (0..8u32)
        .map(|n| unsafe {
            let b: u32 =
                crate::framework::game_read(crate::race::objects::CAR_BLOCKS_RVA + 4 * n as usize);
            (b + 0xB0, 4)
        })
        .collect();
    let mut kept = blocks.clone();
    kept.extend([at(rva(0x52A910)), at(rva(0x52A920)), at(PLAYERS_RVA)]);
    let saved = capture(&kept);
    for (one, chosen, players, out) in [
        (1u32, 3u32, 2u32, 0b0000_1000u32),
        (1, 3, 2, 0),
        (0, 0, 0, 0xFF),
        (0, 0, 2, 0b11),
        (0, 0, 2, 0b10),
        (0, 0, 8, 0xFF),
        (0, 0, 5, 0b1_1111),
        (0, 0, 5, 0b1_0111),
    ] {
        check_state(
            &mut t,
            &format!("one {one}, car {chosen}, {players} players, out {out:#b}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                for (n, &(a, _)) in blocks.iter().enumerate() {
                    (a as *mut u32).write_unaligned((out >> n) & 1);
                }
                game_write(rva(0x52A910), one);
                game_write(rva(0x52A920), chosen);
                game_write(PLAYERS_RVA, players);
            },
            &[],
            &rust,
        );
    }
    restore(&kept, &saved);
    t.finish()
}
