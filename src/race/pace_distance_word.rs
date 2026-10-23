//! `race::pace_distance_word` (0x441C78): a pace note group's first word, for the distance
//! before it: eax (16.16) >> 16 / 10 (by `div`, `mul`, `div`), at most 40, its word from
//! `byte_614D38`; unless 0, it goes into the text (`byte_61BE18`) at edi and edi moves on.
//!
//! In:  eax the distance, edi the text's place
//! Out: edi (+1 when a word was written); the rest kept (`pusha` / `popa`, the dead load
//!      of `dword_615D6C[word]` into eax popped); no flags read
//!
//! Caller: `race::init_pace_notes`. See `race/pace_notes.rs`.
//! Refuses a place outside the pace tables (the original writes past the text).
//! Self-test: the original and the Rust version on the game's tables (put back): distances
//! of every word, below and past 40 tens, places in the text.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::{EDI, Regs};
use crate::race::pace_notes::Machine;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

/// The Rust version.
pub fn pace_distance_word(r: &Regs) -> Result<Left, crate::errors::Refused> {
    let mut m = Machine::load(r, crate::race::pace_notes::game_course());
    m.distance_word()?;
    m.commit();
    Ok(Left::nothing().with(EDI, m.r.edi))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &pace_distance_word)
}
