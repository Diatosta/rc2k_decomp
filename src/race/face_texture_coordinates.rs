//! `race::face_texture_coordinates` (0x453267): the first triangle's texture working values (size word,
//! mapping, the four corners' u, v; see `face_texture_coordinates` in
//! `race/course_faces.rs`).
//!
//! In:  nothing (`dword_63DA2C`, `dword_63DA30`, the corners)
//! Out: eax the size word, ebx the texture record (a `mov` / `call` follows)
//!
//! Caller: `sub_452C6F`. Shadow mode: by state. Self-test: the race's textures, each
//! order, the record's size and mapping set for the cases (put back), corners on test
//! scene points (one before them for a missing fourth); skipped without a race.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{EAX, EBX, Regs};
use crate::race::course_faces::{FIRST_SET, face_texture_coordinates};
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(r: &Regs) -> Result<Left, Refused> {
    let out = unsafe { face_texture_coordinates(&FIRST_SET, r) };
    Ok(Left::nothing().with(EAX, out.eax).with(EBX, out.ebx))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
