//! `scene::set_texture_levels` (0x422426): a run of texture slots given a level and a tag.
//!
//! In:  ecx = how many, ebx = the first slot, eax = the level (edx with `dword_5CDADC`
//!      set), ebp = the tag
//! Out: ebx past the run, ecx 0; the rest preserved
//!
//! See `scene/texture_levels.rs`. 41 calls (`scene::init_texture_levels`, `sub_422457`).
//! Refuses no slots or slots past 0x400. Self-test: the original and the Rust version on
//! the game's tables, put back afterwards.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{EBX, ECX, Regs};
use crate::scene::texture_levels::*;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(r: &Regs) -> Result<Left, Refused> {
    let g = unsafe { set_texture_levels(r) }?;
    Ok(Left::nothing().with(EBX, g.ebx).with(ECX, g.ecx))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
