//! `menu::first_player_unplaced` (0x4A4B45): the first player's entry with a group (+0x10
//! not 0) in standings table B from edi on, counting the computer drivers with a group
//! before it; none: the fatal error 0x8070A0AE "Unable to locate player
//! (find1stplayerinochamp)".
//!
//! In:  edi the first entry
//! Out: edx its +0x70, ecx its player (+4), edi the entry; the rest kept
//!
//! Callers: the event code. Refuses a table without its end entry, before anything;
//! raises the fatal error as the original. Self-test: `standings::test_first_player`.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, hook_asm, rva};
use crate::menu::standings::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{Region, dispatch_state};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "menu::first_player_unplaced",
    rva(0x4A4B45),
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

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn first_player_unplaced(entry: &Regs) -> Result<Regs, Refused> {
    // Converted: `rc2k_core::menu::standings::first_player` (any group), ecx the player.
    let found = crate::menu::standings::first_player(
        &Regs {
            ebx: rc2k_core::menu::standings::ANY_GROUP,
            ..*entry
        },
        B_ENTRY,
        0x10,
        0x70,
        0x8070A0AE,
        "Unable to locate player (find1stplayerinochamp)",
    )?;
    Ok(Regs {
        ecx: get(found.edi + 4),
        ebx: entry.ebx,
        ..found
    })
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { first_player_unplaced(r) }?))
}

fn regions(r: &Regs) -> Vec<Region> {
    let _ = r;
    vec![]
}

extern "C" fn dispatch(r: &mut Regs) {
    let regions = regions(r);
    dispatch_state(&HOOK, r, call_original, &regions, &rust);
}

fn self_test() -> (usize, usize) {
    test_first_player(
        &HOOK,
        call_original,
        B_ENTRY,
        0x10,
        false,
        0x8070A0AE,
        &rust,
    )
}
