//! `install::choice_back` (0x414826): an item of choices (kind 2) moved to the choice before
//! the chosen one that is not marked (bits 6, 7 of its second byte), round from the first
//! to the last; none: nothing changes.
//!
//! In:  esi the item (+1 the choices, +8 the chosen byte, +0xC the choices, 2 bytes each)
//! Out: edx the chosen byte's address, ebx the table, ecx the choice reached (byte); zero
//!      set, carry clear (`cmp eax, eax`)
//! The step down is `dec cl; jns`: from 0x80 .. 0xFF (and 0) it goes to the last choice.
//! A chosen byte past the choices with every choice marked never ends: refused (the
//! original goes round for ever). Callers `install::key_up`, `install::key_left`. Shadow
//! mode compares. Self-test: items of the test's against the original.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.
//! Conversion phase 3 (2026-10-23): plain Rust with typed arguments; the registers above
//! are the original's.

use crate::errors::Refused;
use crate::install::installer::{rb, rd, wb};
use rc2k_core::install::items::{self, MARKS};

/// An item of choices' chosen byte; and whether choice `cl` is free.
pub fn choice_places(item: u32) -> (u32, impl Fn(u8) -> bool + use<>) {
    let table = rd(item + 0xC);
    let free = move |cl: u8| rb(table.wrapping_add(2 * cl as u32 + 1)) & MARKS == 0;
    (rd(item + 8), free)
}

/// The choice reached stored, when it changed.
pub fn store_choice(chosen_at: u32, chosen: u8, cl: u8) {
    if cl != chosen {
        wb(chosen_at, cl);
    }
}

pub fn choice_back(item: u32) -> Result<(), Refused> {
    let (chosen_at, free) = choice_places(item);
    let chosen = rb(chosen_at);
    // Converted: `rc2k_core::install::items::choice_back`.
    let cl = items::choice_back(chosen, || rb(item + 1), free)?;
    store_choice(chosen_at, chosen, cl);
    Ok(())
}
