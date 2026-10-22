//! `menu::first_player_a` (0x4A2023): the first player's entry of group ebx (0x63: any group but
//! 0) in standings table A from edi on (`menu::standings::first_player`); none: the
//! fatal error 0x8070A0A0 "Unable to locate player (find1stplayer)".
//!
//! In:  edi the first entry, ebx the group
//! Out: ecx the group's computer drivers before it, edx its +0x44, edi the entry; the
//!      rest kept
//!
//! Callers: the event code. Refuses a table without its end entry, before anything;
//! raises the fatal error as the original. Self-test: `standings::test_first_player`.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, hook_asm, rva};
use crate::menu::standings::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{Region, dispatch_state, state_entry};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::first_player_a", rva(0x4A2023), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn first_player_a(entry: &Regs) -> Result<Regs, Refused> {
    first_player(
        entry,
        A_ENTRY,
        0x1C,
        0x44,
        0x8070A0A0,
        "Unable to locate player (find1stplayer)",
    )
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { first_player_a(r) }?))
}

fn regions(r: &Regs) -> Vec<Region> {
    let _ = r;
    vec![]
}

extern "C" fn dispatch(r: &mut Regs) {
    let regions = regions(r);
    dispatch_state(&HOOK, r, call_original, &regions, &rust);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}

fn self_test() -> (usize, usize) {
    test_first_player(&HOOK, call_original, A_ENTRY, 0x1C, true, 0x8070A0A0, &rust)
}
