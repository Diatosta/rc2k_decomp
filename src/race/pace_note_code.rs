//! `race::pace_note_code` (0x441A37): the pace note code of the face at esi in the
//! current slot: the byte at esi + `dword_614D14[dword_614D34]` (see `race/pace_notes.rs`).
//!
//! In:  esi the face, `dword_614D34` the slot
//! Out: eax the code (`movzx`); the rest kept (ebx pushed and popped); no flags read
//!
//! Callers: `race::init_pace_notes` (twice), `race::pace_note_calls`.
//! Refuses a slot past 7 and a face outside the course memory (the original reads past its
//! table or the course).
//! Self-test: the original and the Rust version on a test stage (`pace_notes::TestStage`),
//! every slot of several faces.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::{EAX, Regs};
use crate::race::pace_notes::{Course, Machine};
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

/// The Rust version on a course.
pub fn pace_note_code(r: &Regs, course: Course) -> Result<Left, crate::errors::Refused> {
    let mut m = Machine::load(r, course);
    m.code()?;
    Ok(Left::nothing().with(EAX, m.r.eax))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &|r| {
        pace_note_code(r, crate::race::pace_notes::game_course())
    })
}
