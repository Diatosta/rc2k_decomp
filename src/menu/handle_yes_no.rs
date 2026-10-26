//! `menu::handle_yes_no` (0x470D2E): on a screen that asks a yes/no question, lets a key
//! answer it.
//!
//! In:  eax = the key
//! Out: every register preserved
//! Only while `dword_7554CC` is 1. A key that `input::yes_no_for_key` takes for yes stores 1
//! in `dword_75524C`, one for no stores 0; either also sets `dword_758806` to 1 and leaves
//! the screen with `menu::go_back`. One caller, `menu::handle_keys`.
//! Refuses what the yes/no functions refuse.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::Regs;
use crate::input::yes_no_for_key::yes_no_for_key;
use crate::input::yes_no_key::{NO, YES, yes_no_regions};
use crate::menu::leaving::{question, with_question};
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, state_entry};

// ---- game data ----

pub use crate::menu::leaving::{ANSWER_RVA, ANSWERED_RVA as LEAVING_RVA, ASKING_RVA};

/// What the function may write.
pub fn handle_yes_no_regions() -> Vec<Region> {
    [
        &yes_no_regions()[..],
        &[at(ANSWER_RVA), at(LEAVING_RVA)],
        &crate::menu::go_back::regions(),
    ]
    .concat()
}

// ---- Rust version ----

pub unsafe fn handle_yes_no(eax: u32) -> Result<(), BadInput> {
    // Converted: `rc2k_core::menu::leaving::Question::answer_key`.
    if !question().asks() {
        return Ok(());
    }
    let said = match unsafe { yes_no_for_key(eax) }? {
        YES => Some(true),
        NO => Some(false),
        _ => None,
    };
    with_question(|q, l| q.answer_key(l, said));
    Ok(())
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { handle_yes_no(r.eax) }?;
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
