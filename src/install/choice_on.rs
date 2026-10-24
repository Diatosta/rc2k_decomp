//! `install::choice_on` (0x414848): an item of choices (kind 2) moved to the choice after
//! the chosen one that is not marked (bits 6, 7 of its second byte), round from the last to
//! the first; none: nothing changes.
//!
//! In:  esi the item (+1 the choices, +8 the chosen byte, +0xC the choices, 2 bytes each)
//! Out: edx the chosen byte's address, ebx the table, ecx the choice reached (byte); zero
//!      set, carry clear (`cmp eax, eax`)
//! The step up is `inc cl; cmp cl, [esi+1]; jb` (else 0). A chosen byte past the choices
//! with every choice marked never ends: refused (the original goes round for ever).
//! Callers `install::key_down`, `install::key_right`. Shadow mode compares. Self-test:
//! items of the test's against the original.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.
//! Conversion phase 3 (2026-10-23): plain Rust with typed arguments; the registers above
//! are the original's.

use crate::errors::Refused;
use crate::install::choice_back::{choice_places, store_choice};
use crate::install::installer::rb;

pub fn choice_on(item: u32) -> Result<(), Refused> {
    let (chosen_at, free) = choice_places(item);
    let chosen = rb(chosen_at);
    // Converted: `rc2k_core::install::items::choice_on`.
    let cl = rc2k_core::install::items::choice_on(chosen, || rb(item + 1), free)?;
    store_choice(chosen_at, chosen, cl);
    Ok(())
}
